"""Bounded, read-only assertions for native regional emulator fixtures.

``check(connection, scan_id, fixtures)`` returns harness-compatible records with
``check``, ``passed`` and optional ``detail`` keys. Pass the ``fixtures`` member of
seed_regional.seed's report and the exact native scan ID. Only direct observations
from that scan are eligible. No credential access, AWS calls or database writes
occur here. Region defaults to the harness's us-east-1 and can be passed explicitly.

Expected IDs and relationship targets come from creation responses or the seeder's
request values. Legacy reports without the newer creation-reference maps retain
the documented two-cluster/two-child creation order; partial/malformed fixture
sets fail validation before any positional association is made. Target-health IDs
are derived from the registered group, instance and port, not observed SQL data.
EKS legacy nodegroups are matched on both created cluster name and nodegroup name.

A successful snapshot cannot prove failure isolation, IAM authorization or data-
plane reachability. ECS tasks can already be STOPPED; task status is reported as a
limitation and does not manufacture evidence for a populated RUNNING task list.
Wire operation coverage belongs to the harness request matrix, not this module.
"""

from __future__ import annotations

import collections
import json
import sqlite3
from typing import Any

MAX_RESOURCES = 1_000
MAX_EDGES = 2_000
MAX_SCOPES = 1_000
MAX_CHECKS = 2_048
MAX_FIXTURE_BYTES = 512 * 1_024
MAX_PROPERTY_BYTES = 2 * 1_024 * 1_024
MAX_TOTAL_PROPERTY_BYTES = 16 * 1_024 * 1_024
MAX_DETAIL_BYTES = 4_096
MAX_ID_BYTES = 2_048


def _detail(value: Any) -> Any:
    if isinstance(value, set):
        value = sorted(value)
    encoded = json.dumps(value, sort_keys=True, ensure_ascii=False)
    if len(encoded.encode()) > MAX_DETAIL_BYTES:
        return {"detail_truncated": True, "encoded_bytes": len(encoded.encode())}
    return value


def _rows(connection: sqlite3.Connection, query: str, parameters: tuple, cap: int) -> list[dict]:
    cursor = connection.execute(query, parameters)
    names = [column[0] for column in cursor.description]
    rows = []
    payload_bytes = 0
    for index, values in enumerate(cursor):
        if index == cap:
            raise ValueError(f"regional assertion input exceeds row cap {cap}")
        payload_bytes += sum(len(value.encode()) for value in values if isinstance(value, str))
        if payload_bytes > MAX_TOTAL_PROPERTY_BYTES:
            raise ValueError("regional assertion input exceeds the aggregate byte cap")
        rows.append(dict(zip(names, values)))
    return rows


def _identifier(value: Any) -> bool:
    return isinstance(value, str) and 0 < len(value.encode()) <= MAX_ID_BYTES


def _validate_fixtures(fixtures: dict, emit) -> bool:
    """Fail closed instead of vacuous success when creation was incomplete."""
    valid = True

    def require(label, condition, detail):
        nonlocal valid
        emit("regional:fixtures:" + label, condition, detail)
        valid = valid and condition

    def sequence(container, key, length, label=None):
        values = container.get(key) if isinstance(container, dict) else None
        good = isinstance(values, list) and len(values) == length and all(_identifier(v) for v in values)
        if good:
            good = len(set(values)) == len(values)
        require(label or key, good, {"expected_count": length, "observed_count": len(values) if isinstance(values, list) else None})
        return values if good else []

    network = fixtures.get("ec2", {})
    for key in ("vpc_ids", "vpc_cidr_blocks", "subnet_ids", "security_group_ids", "network_interface_ids", "volume_ids", "route_table_ids", "internet_gateway_ids", "nat_gateway_ids", "vpc_endpoint_ids", "network_acl_ids"):
        sequence(network, key, 2)
    sequence(network, "instance_ids", 6)
    sequence(network, "vpc_peering_connection_ids", 1)
    require("image_id", isinstance(network, dict) and _identifier(network.get("image_id")), "created instances must reference an observed image ID")
    roles = fixtures.get("roles", {})
    for role in ("lambda", "ecs", "eks_cluster", "eks_node"):
        require("role:" + role, isinstance(roles, dict) and _identifier(roles.get(role)), "role ARN must be recorded by fixture creation")
    sequence(fixtures.get("lambda", {}), "function_arns", 2)
    sequence(fixtures.get("dynamodb", {}), "table_names", 2)
    sequence(fixtures.get("ecs", {}), "task_definition_arns", 4)
    sequence(fixtures.get("elbv2", {}), "target_group_arns", 2)
    for service, key, children in (
        ("ecs", "clusters", {"service_arns": 2, "task_arns": 2}),
        ("eks", "clusters", {"nodegroups": 2}),
        ("elbv2", "load_balancers", {"listeners": 2, "rule_arns": 4}),
    ):
        container = fixtures.get(service, {})
        parents = container.get(key) if isinstance(container, dict) else None
        shape = isinstance(parents, list) and len(parents) == 2 and all(isinstance(p, dict) and _identifier(p.get("arn")) for p in parents)
        if shape:
            shape = len({p["arn"] for p in parents}) == len(parents)
        require(service + ":" + key, shape, "two distinct created parent ARNs are required")
        if not shape:
            continue
        for index, parent in enumerate(parents):
            if service == "eks":
                require(f"eks:{index}:name", _identifier(parent.get("name")), "created cluster name is required")
            for child, size in children.items():
                sequence(parent, child, size, f"{service}:{index}:{child}")
    return valid


def check(connection: sqlite3.Connection, scan_id: str, fixtures: dict | None, *, region: str = "us-east-1") -> list[dict]:
    """Check one native scan against actual regional creation fixtures."""
    checks: list[dict] = []

    def emit(name, passed, detail=None):
        if len(checks) >= MAX_CHECKS:
            raise ValueError("regional assertion check budget exceeded")
        row = {"check": name, "passed": bool(passed)}
        if detail is not None:
            row["detail"] = _detail(detail)
        checks.append(row)

    try:
        if not _identifier(scan_id) or not _identifier(region):
            raise ValueError("scan ID and region must be bounded nonempty strings")
        if not isinstance(fixtures, dict) or len(json.dumps(fixtures).encode()) > MAX_FIXTURE_BYTES:
            raise ValueError("regional fixture report is missing or exceeds the fixture byte cap")
        if not _validate_fixtures(fixtures, emit):
            return checks
        resource_rows = _rows(connection,
            "SELECT id,kind,name,source,account_id,region,"
            "CASE WHEN length(CAST(properties AS BLOB))<=? THEN properties ELSE NULL END AS properties "
            "FROM scan_resources "
            "WHERE scan_id=? AND source='direct' ORDER BY kind,id LIMIT ?",
            (MAX_PROPERTY_BYTES, scan_id, MAX_RESOURCES + 1), MAX_RESOURCES)
        edge_rows = _rows(connection,
            "SELECT from_id,to_id,relation FROM scan_edges "
            "WHERE scan_id=? AND source='direct' ORDER BY from_id,to_id,relation LIMIT ?",
            (scan_id, MAX_EDGES + 1), MAX_EDGES)
        scopes = _rows(connection,
            "SELECT collector,parent_id,last_page,complete,failed FROM scope_checkpoints "
            "WHERE scan_id=? AND source='direct' ORDER BY collector,parent_id LIMIT ?",
            (scan_id, MAX_SCOPES + 1), MAX_SCOPES)
        resources = {}
        property_bytes = 0
        for row in resource_rows:
            value = row["properties"]
            if not isinstance(value, str) or len(value.encode()) > MAX_PROPERTY_BYTES:
                raise ValueError("resource properties missing or exceed the per-resource byte cap")
            property_bytes += len(value.encode())
            if property_bytes > MAX_TOTAL_PROPERTY_BYTES:
                raise ValueError("resource properties exceed the total byte cap")
            row["properties"] = json.loads(value)
            if not isinstance(row["properties"], dict):
                raise ValueError("resource properties must be a JSON object")
            resources[row["id"]] = row
        edges = {(r["from_id"], r["relation"], r["to_id"]) for r in edge_rows}
        _assert_snapshot(resources, edges, scopes, fixtures, region, emit)
        statuses = collections.Counter(
            resources[identity]["properties"].get("last_status") or "unknown"
            for cluster in fixtures["ecs"]["clusters"]
            for identity in cluster["task_arns"] if identity in resources
        )
        emit("regional:evidence-limit:task-status", True, {
            "observed_last_status_counts": dict(sorted(statuses.items())),
            "limitation": "Task creation and ListTasks coverage do not establish that a RUNNING list contained tasks; stopped tasks remain inventory evidence.",
        })
        emit("regional:evidence-limit:failure-isolation", True, {
            "limitation": "Snapshot assertions do not inject failures or certify parent retention during child failure. Use the separate synthetic regression suite.",
        })
    except (sqlite3.Error, TypeError, ValueError, KeyError, IndexError, RecursionError) as error:
        # Malformed inputs/caps are failing assertions, never empty-success reports.
        if len(checks) >= MAX_CHECKS:
            checks.pop()
        emit("regional:input-validation", False, {"error_type": type(error).__name__, "error": str(error)[:512]})
    return checks


def _assert_snapshot(resources, edges, scopes, fixtures, region, emit):
    def check(label, actual, expected=True, category='field'):
        emit('regional:' + label, actual == expected, {'category': category, 'actual': actual, 'expected': expected})

    def resource(identity, kind):
        row = resources.get(identity)
        check('fixture:' + identity, None if row is None else row['kind'], kind, 'identity')
        if row is not None:
            check('direct-source:' + identity, row['source'], 'direct', 'source')
            check('account:' + identity, row['account_id'], '000000000000', 'source')
            check('region:' + identity, row['region'], region, 'source')
        return row

    def properties(row):
        return {} if row is None else row['properties']

    def fields(row, expected):
        if row is None:
            return
        for key, value in expected.items():
            check(row['id'] + ':' + key, row['properties'].get(key), value)

    def array(row, key, expected):
        if row is not None:
            check(row['id'] + ':' + key, sorted(row['properties'].get(key) or []), sorted(expected))

    def edge(row, relation, target):
        if row is not None:
            check(row['id'] + ':' + relation + ':' + target, (row['id'], relation, target) in edges, category='relationship')

    def ec2(path, identity):
        return f'arn:aws:ec2:{region}:000000000000:{path}/{identity}'

    network = fixtures['ec2']
    network_map = [
        ('vpc_ids','vpc','aws_vpc','vpc_id'),
        ('subnet_ids','subnet','aws_vpc_subnet','subnet_id'),
        ('security_group_ids','security-group','aws_vpc_security_group','group_id'),
        ('network_interface_ids','network-interface','aws_ec2_network_interface','network_interface_id'),
        ('volume_ids','volume','aws_ebs_volume','volume_id'),
        ('instance_ids','instance','aws_ec2_instance','instance_id'),
        ('route_table_ids','route-table','aws_vpc_route_table','route_table_id'),
        ('internet_gateway_ids','internet-gateway','aws_vpc_internet_gateway','internet_gateway_id'),
        ('nat_gateway_ids','natgateway','aws_vpc_nat_gateway','nat_gateway_id'),
        ('vpc_endpoint_ids','vpc-endpoint','aws_vpc_endpoint','vpc_endpoint_id'),
        ('network_acl_ids','network-acl','aws_vpc_network_acl','network_acl_id'),
        ('vpc_peering_connection_ids','vpc-peering-connection','aws_vpc_peering_connection','vpc_peering_connection_id'),
    ]
    regional_kinds = {entry[2] for entry in network_map} | {
        'aws_dynamodb_table','aws_lambda_function','aws_ecs_cluster','aws_ecs_service','aws_ecs_task','aws_ecs_task_definition',
        'aws_eks_cluster','aws_eks_node_group','aws_elbv2_load_balancer','aws_elbv2_listener','aws_elbv2_rule','aws_elbv2_target_group','aws_elbv2_target_health',
    }
    counts = collections.Counter(row['kind'] for row in resources.values())
    for kind in sorted(regional_kinds):
        check('kind-observed:' + kind, counts[kind] > 0, category='coverage')
    for key, path, kind, field in network_map:
        for identity in network[key]:
            row = resource(ec2(path, identity), kind)
            fields(row, {field: identity})
    primary_vpc = network['vpc_ids'][0]
    for key, path, kind, _ in network_map:
        if key in {'subnet_ids','security_group_ids','network_interface_ids','route_table_ids','nat_gateway_ids','vpc_endpoint_ids','network_acl_ids'}:
            for identity in network[key]:
                row = resources.get(ec2(path, identity))
                fields(row, {'vpc_id': primary_vpc})
                edge(row, 'IN_VPC', ec2('vpc', primary_vpc))
    for index, identity in enumerate(network['vpc_ids']):
        fields(resources.get(ec2('vpc', identity)), {'cidr_block': network['vpc_cidr_blocks'][index]})
    for index, identity in enumerate(network['subnet_ids']):
        fields(resources.get(ec2('subnet', identity)), {'availability_zone': region + ('a','b')[index], 'cidr_block': '.'.join(network['vpc_cidr_blocks'][0].split('.')[:2]) + f'.{index+1}.0/24'})
    for index, identity in enumerate(network['security_group_ids']):
        row = resources.get(ec2('security-group', identity))
        perms = properties(row).get('ip_permissions', [])
        check(identity + ':declared-http-ingress', any(p.get('ip_protocol')=='tcp' and p.get('from_port')==80 and p.get('to_port')==80 and any(v.get('cidr_ip')=='0.0.0.0/0' for v in p.get('ipv4_ranges',[])) for p in perms))
    for index, identity in enumerate(network['network_interface_ids']):
        row = resources.get(ec2('network-interface', identity))
        fields(row, {'subnet_id': network['subnet_ids'][index]})
        array(row, 'security_group_ids', [network['security_group_ids'][index]])
        edge(row, 'IN_SUBNET', ec2('subnet',network['subnet_ids'][index]))
        edge(row, 'USES_SECURITY_GROUP', ec2('security-group',network['security_group_ids'][index]))
    for identity in network['instance_ids']:
        row = resources.get(ec2('instance',identity))
        fields(row, {'instance_type':'t3.micro','image_id':network['image_id'],'vpc_id':primary_vpc,'subnet_id':network['subnet_ids'][0]})
        array(row,'security_group_ids',[network['security_group_ids'][0]])
        edge(row,'IN_VPC',ec2('vpc',primary_vpc)); edge(row,'IN_SUBNET',ec2('subnet',network['subnet_ids'][0])); edge(row,'USES_SECURITY_GROUP',ec2('security-group',network['security_group_ids'][0]))
    for index, identity in enumerate(network['volume_ids']):
        row = resources.get(ec2('volume',identity))
        fields(row, {'size':1,'volume_type':'gp3','availability_zone':region+('a','b')[index]})
        if index==0:
            edge(row,'ATTACHED_TO',ec2('instance',network['instance_ids'][0]))
        else:
            check(identity+':unattached',properties(row).get('attachments'),[])
    for index, identity in enumerate(network['route_table_ids']):
        row=resources.get(ec2('route-table',identity)); routes=properties(row).get('routes',[])
        check(identity+':default-igw-route',any(r.get('destination_cidr_block')=='0.0.0.0/0' and r.get('gateway_id')==network['internet_gateway_ids'][0] and r.get('state')=='active' for r in routes))
        edge(row,'ASSOCIATED_WITH_SUBNET',ec2('subnet',network['subnet_ids'][index])); edge(row,'ROUTES_TO',ec2('internet-gateway',network['internet_gateway_ids'][0]))
    for index,identity in enumerate(network['internet_gateway_ids']):
        edge(resources.get(ec2('internet-gateway',identity)),'ATTACHED_TO_VPC',ec2('vpc',network['vpc_ids'][index]))
    for index,identity in enumerate(network['nat_gateway_ids']):
        row=resources.get(ec2('natgateway',identity)); fields(row,{'subnet_id':network['subnet_ids'][index],'connectivity_type':'public'}); edge(row,'IN_SUBNET',ec2('subnet',network['subnet_ids'][index]))
        for binding in network.get('nat_gateway_bindings', []):
            if binding['id'] == identity:
                edge(row,'USES_ELASTIC_IP',ec2('elastic-ip',binding['allocation_id']))
                for interface in binding['network_interface_ids']:
                    edge(row,'USES_NETWORK_INTERFACE',ec2('network-interface',interface))
    for index,identity in enumerate(network['vpc_endpoint_ids']):
        row=resources.get(ec2('vpc-endpoint',identity)); fields(row,{'vpc_endpoint_type':'Gateway','service_name':f'com.amazonaws.{region}.'+('s3','dynamodb')[index]}); array(row,'route_table_ids',[network['route_table_ids'][index]]); edge(row,'USES_ROUTE_TABLE',ec2('route-table',network['route_table_ids'][index]))
    for identity in network['network_acl_ids']:
        row=resources.get(ec2('network-acl',identity)); entries=properties(row).get('entries',[])
        check(identity+':declared-http-rule',any(e.get('rule_number')==100 and e.get('protocol')=='6' and e.get('egress') is False and e.get('rule_action')=='allow' and e.get('port_range')=={'from':80,'to':80} for e in entries))
        fields(row,{'is_default':False,'associations':[]})
    for identity in network['vpc_peering_connection_ids']:
        row=resources.get(ec2('vpc-peering-connection',identity)); fields(row,{'status_code':'pending-acceptance','requester_vpc_id':primary_vpc,'accepter_vpc_id':network['vpc_ids'][1]}); edge(row,'REQUESTER_VPC',ec2('vpc',primary_vpc)); edge(row,'ACCEPTER_VPC',ec2('vpc',network['vpc_ids'][1]))
    for identity in fixtures['lambda']['function_arns']:
        row=resource(identity,'aws_lambda_function'); fields(row,{'runtime':'python3.12','handler':'handler.handler','memory_size':128,'role':fixtures['roles']['lambda']}); edge(row,'ASSUMES_ROLE',fixtures['roles']['lambda'])
        for key,relation,path in [('subnet_ids','USES_SUBNET','subnet'),('security_group_ids','USES_SECURITY_GROUP','security-group')]:
            array(row,key,network[key])
            for child in network[key]: edge(row,relation,ec2(path,child))
    for name in fixtures['dynamodb']['table_names']:
        row=resource(f'arn:aws:dynamodb:{region}:000000000000:table/{name}','aws_dynamodb_table'); fields(row,{'table_name':name,'billing_mode':'PAY_PER_REQUEST','key_schema':[{'attribute_name':'id','key_type':'HASH'}],'attribute_definitions':[{'attribute_name':'id','attribute_type':'S'}]})
    for cluster_index, cluster in enumerate(fixtures['ecs']['clusters']):
        row=resource(cluster['arn'],'aws_ecs_cluster'); fields(row,{'cluster_arn':cluster['arn'],'status':'ACTIVE'})
        for child_index, identity in enumerate(cluster['service_arns']):
            definition = cluster.get('service_task_definitions', {}).get(identity, fixtures['ecs']['task_definition_arns'][cluster_index * 2 + child_index])
            row=resource(identity,'aws_ecs_service'); fields(row,{'cluster_arn':cluster['arn'],'launch_type':'FARGATE','desired_count':0,'task_definition':definition}); edge(row,'IN_CLUSTER',cluster['arn']); edge(row,'USES_TASK_DEFINITION',definition)
            check(identity+':exact-definition-reference',properties(row).get('task_definition') in fixtures['ecs']['task_definition_arns'])
            for key,relation,path in [('subnet_ids','USES_SUBNET','subnet'),('security_group_ids','USES_SECURITY_GROUP','security-group')]:
                array(row,key,network[key])
                for child in network[key]: edge(row,relation,ec2(path,child))
        for child_index, identity in enumerate(cluster['task_arns']):
            definition = cluster.get('task_definitions', {}).get(identity, fixtures['ecs']['task_definition_arns'][cluster_index * 2 + child_index])
            row=resource(identity,'aws_ecs_task'); fields(row,{'cluster_arn':cluster['arn'],'launch_type':'FARGATE','cpu':'256','memory':'512','task_definition_arn':definition}); edge(row,'IN_CLUSTER',cluster['arn']); edge(row,'USES_TASK_DEFINITION',definition)
            check(identity+':exact-definition-reference',properties(row).get('task_definition_arn') in fixtures['ecs']['task_definition_arns'])
    for identity in fixtures['ecs']['task_definition_arns']:
        row=resource(identity,'aws_ecs_task_definition'); fields(row,{'revision':1,'cpu':'256','memory':'512','network_mode':'awsvpc','task_role_arn':fixtures['roles']['ecs'],'execution_role_arn':fixtures['roles']['ecs']}); array(row,'requires_compatibilities',['FARGATE']); edge(row,'USES_TASK_ROLE',fixtures['roles']['ecs']); edge(row,'USES_EXECUTION_ROLE',fixtures['roles']['ecs'])
    for cluster in fixtures['eks']['clusters']:
        row=resource(cluster['arn'],'aws_eks_cluster'); fields(row,{'role_arn':fixtures['roles']['eks_cluster'],'vpc_id':primary_vpc,'endpoint_public_access':False,'endpoint_private_access':True}); edge(row,'ASSUMES_ROLE',fixtures['roles']['eks_cluster']); edge(row,'IN_VPC',ec2('vpc',primary_vpc))
        for key,relation,path in [('subnet_ids','USES_SUBNET','subnet'),('security_group_ids','USES_SECURITY_GROUP','security-group')]:
            array(row,key,network[key])
            for child in network[key]: edge(row,relation,ec2(path,child))
        for name in cluster['nodegroups']:
            found=[r for r in resources.values() if r['kind']=='aws_eks_node_group' and r['properties'].get('cluster_name')==cluster['name'] and r['properties'].get('nodegroup_name')==name]
            check(cluster['name']+'/'+name+':one-nodegroup',len(found),1,'identity')
            if len(found)==1:
                observed = found[0]['id']
                expected = cluster.get('nodegroup_arns', {}).get(name)
                prefix = f"arn:aws:eks:{region}:000000000000:nodegroup/{cluster['name']}/{name}/"
                check(cluster['name']+'/'+name+':arn-scope', observed.startswith(prefix) and len(observed)>len(prefix), category='identity')
                if expected is not None: check(cluster['name']+'/'+name+':created-arn',observed,expected,'identity')
                row=resource(expected or observed,'aws_eks_node_group'); fields(row,{'node_role':fixtures['roles']['eks_node'],'desired_size':0,'min_size':0,'max_size':1}); array(row,'subnets',network['subnet_ids']); edge(row,'IN_CLUSTER',cluster['arn']); edge(row,'ASSUMES_ROLE',fixtures['roles']['eks_node'])
                for subnet in network['subnet_ids']: edge(row,'USES_SUBNET',ec2('subnet',subnet))
    for index,lb in enumerate(fixtures['elbv2']['load_balancers']):
        row=resource(lb['arn'],'aws_elbv2_load_balancer'); fields(row,{'type':'application','scheme':'internet-facing','vpc_id':primary_vpc}); edge(row,'IN_VPC',ec2('vpc',primary_vpc))
        for key,relation,path in [('subnet_ids','USES_SUBNET','subnet'),('security_group_ids','USES_SECURITY_GROUP','security-group')]:
            array(row,key,network[key])
            for child in network[key]: edge(row,relation,ec2(path,child))
        group=lb.get('target_group_arn') or fixtures['elbv2']['target_group_arns'][index]
        for listener_index,identity in enumerate(lb['listeners']):
            row=resource(identity,'aws_elbv2_listener'); fields(row,{'load_balancer_arn':lb['arn'],'protocol':'HTTP','port':lb.get('listener_ports',{}).get(identity,(80,8080)[listener_index])}); array(row,'default_target_group_arns',[group]); edge(row,'LISTENER_OF',lb['arn']); edge(row,'FORWARDS_TO',group)
        for rule_index,identity in enumerate(lb['rule_arns']):
            listener=lb.get('rule_listeners',{}).get(identity,lb['listeners'][rule_index//2]); row=resource(identity,'aws_elbv2_rule'); fields(row,{'listener_arn':listener,'is_default':False,'priority':str(lb.get('rule_priorities',{}).get(identity,(10,20)[rule_index%2]))}); array(row,'target_group_arns',[group]); edge(row,'RULE_OF',listener); edge(row,'FORWARDS_TO',group)
        row=resource(group,'aws_elbv2_target_group'); fields(row,{'vpc_id':primary_vpc,'protocol':'HTTP','port':80,'target_type':'instance'}); edge(row,'IN_VPC',ec2('vpc',primary_vpc)); edge(row,'TARGET_GROUP_OF',lb['arn'])
        for target in network['instance_ids'][:2]:
            identity='aws-elbv2-target-health:'+json.dumps([group,target,80,None,None],separators=(',',':'))
            health=resource(identity,'aws_elbv2_target_health'); fields(health,{'target_group_arn':group,'target_id':target,'target_port':80,'target_type':'instance'}); edge(row,'HAS_TARGET',identity); edge(health,'TARGETS',ec2('instance',target))

    regional_scopes=[scope for scope in scopes if scope['collector'] in regional_kinds or scope['collector'].startswith(('aws_ecs_','aws_eks_','aws_elbv2_'))]
    roots = [entry[2] for entry in network_map] + [
        'aws_dynamodb_table', 'aws_lambda_function', 'aws_ecs_cluster',
        'aws_eks_cluster', 'aws_elbv2_load_balancer', 'aws_elbv2_target_group',
    ]
    for collector in roots:
        selected = [s for s in regional_scopes if s['collector']==collector and s['parent_id'] is None]
        check(collector+':root-scope-present', len(selected), 1, 'pagination')
    for scope in regional_scopes:
        label=scope['collector']+':'+str(scope['parent_id'])
        check(label+':scope-complete',scope['complete'],1,'pagination')
        check(label+':scope-no-failure',scope['failed'],0,'pagination')
    for collector,minimum in [('aws_ec2_instance',2),('aws_dynamodb_table',2),('aws_lambda_function',2),('aws_ecs_cluster',2),('aws_ecs_task_definition_list',4),('aws_eks_cluster',2),('aws_elbv2_load_balancer',2),('aws_elbv2_target_group',2)]:
        selected=[s for s in regional_scopes if s['collector']==collector]
        check(collector+':continuation-exercised',any(s['last_page']>=minimum for s in selected),category='pagination')
    for collector,parents,minimum in [
        ('aws_ecs_service',[c['arn'] for c in fixtures['ecs']['clusters']],2),
        ('aws_eks_node_group',[c['arn'] for c in fixtures['eks']['clusters']],2),
        ('aws_elbv2_listener',[c['arn'] for c in fixtures['elbv2']['load_balancers']],2),
        ('aws_elbv2_rule',[a for c in fixtures['elbv2']['load_balancers'] for a in c['listeners']],3),
    ]:
        for parent in parents:
            selected=[s for s in regional_scopes if s['collector']==collector and s['parent_id']==parent]
            check(collector+':'+parent+':continuation-exercised',len(selected)==1 and selected[0]['last_page']>=minimum,category='pagination')
    for cluster in fixtures['ecs']['clusters']:
        parents = {cluster['arn']+'#desired='+status for status in ('RUNNING', 'STOPPED')}
        selected = [s for s in regional_scopes if s['collector']=='aws_ecs_task' and s['parent_id'] in parents]
        check(cluster['arn']+':both-task-status-scopes', len(selected), 2, 'pagination')
        check(cluster['arn']+':task-continuation-exercised', any(s['last_page']>=2 for s in selected), category='pagination')

