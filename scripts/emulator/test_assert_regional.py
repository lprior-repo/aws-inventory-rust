"""Exercise regional assertions against a completed real emulator evidence set.

Run with --evidence-dir artifacts/emulator-v04-second (or another completed run).
The original database is opened read-only and copied to memory. All deliberately
invalid evidence is inserted only into that copy, under rolled-back savepoints.
No AWS or emulator call occurs. Negative checks are assertion validation evidence,
not successful AWS collector calls and not live failure-isolation tests.
"""

from __future__ import annotations

import argparse
import copy
import json
import pathlib
import sqlite3

import assert_regional


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence-dir", type=pathlib.Path, required=True)
    args = parser.parse_args()
    base=args.evidence_dir.resolve()
    fixtures=json.loads((base/'seed-results.json').read_text())['modules']['seed_regional']['fixtures']
    scan=json.loads((base/'native.json').read_text())['result']['scans'][0]['scan_id']
    original=sqlite3.connect((base/'inventory.sqlite').resolve().as_uri()+'?mode=ro',uri=True)
    db=sqlite3.connect(':memory:')
    original.backup(db)
    original.close()
    results=[]

    def verify(name, condition, detail=None):
        results.append({'case':name,'passed':bool(condition),'detail':detail})

    baseline=assert_regional.check(db,scan,fixtures)
    verify('baseline-real-emulator-database',bool(baseline) and all(r['passed'] for r in baseline), {'checks':len(baseline)})
    verify('baseline-unique-check-identifiers',len({r['check'] for r in baseline})==len(baseline))
    (base/'regional-assertions.json').write_text(json.dumps(baseline,indent=2,sort_keys=True)+'\n')

    for name,value in [('missing-fixture-report',None),('empty-fixture-report',{}),('malformed-fixture-report',{'ec2':'invalid'})]:
        check=assert_regional.check(db,scan,value)
        verify(name, bool(check) and any(not r['passed'] for r in check))
    check=assert_regional.check(db,'different-scan-with-no-fixture-observations',fixtures)
    verify('scan-isolation',any(not r['passed'] and ':fixture:' in r['check'] for r in check))

    instance='arn:aws:ec2:us-east-1:000000000000:instance/'+fixtures['ec2']['instance_ids'][0]
    service=fixtures['ecs']['clusters'][0]['service_arns'][0]
    wrong_definition=fixtures['ecs']['task_definition_arns'][1]
    edge=db.execute("SELECT from_id,to_id,relation FROM edge_observations WHERE scan_id=? AND from_id=? AND relation='IN_VPC' LIMIT 1",(scan,instance)).fetchone()

    def altered(name, sql, params, require):
        db.execute('SAVEPOINT test_mutation')
        try:
            db.execute(sql,params)
            check=assert_regional.check(db,scan,fixtures)
            failures=[r for r in check if not r['passed']]
            verify(name,bool(failures) and require(failures), {'failing_checks':len(failures)})
        finally:
            db.execute('ROLLBACK TO test_mutation')
            db.execute('RELEASE test_mutation')

    altered('missing-created-resource', 'DELETE FROM resource_observations WHERE scan_id=? AND resource_id=?', (scan,instance), lambda failed:any(r['check']=='regional:fixture:'+instance for r in failed))
    altered('non-direct-resource-does-not-count', "UPDATE resource_observations SET source='cloud_control',source_priority=2 WHERE scan_id=? AND resource_id=?", (scan,instance), lambda failed:any(r['check']=='regional:fixture:'+instance for r in failed))
    altered('wrong-account-metadata', "UPDATE resource_observations SET account_id='999999999999' WHERE scan_id=? AND resource_id=?", (scan,instance), lambda failed:any(r['check']=='regional:account:'+instance for r in failed))
    altered('missing-created-relationship', 'DELETE FROM edge_observations WHERE scan_id=? AND from_id=? AND to_id=? AND relation=?', (scan,*edge), lambda failed:any(':IN_VPC:' in r['check'] for r in failed))
    altered('non-direct-edge-does-not-count', "UPDATE edge_observations SET source='cloud_control',source_priority=2 WHERE scan_id=? AND from_id=? AND to_id=? AND relation=?", (scan,*edge), lambda failed:any(':IN_VPC:' in r['check'] for r in failed))
    altered('wrong-but-existing-task-revision', "UPDATE resource_observations SET resource_json=json_set(resource_json,'$.properties.task_definition',?) WHERE scan_id=? AND resource_id=?", (wrong_definition,scan,service), lambda failed:any(r['check']=='regional:'+service+':task_definition' for r in failed))
    altered('missing-unpaginated-root-scope', "DELETE FROM scope_checkpoints WHERE scan_id=? AND collector='aws_vpc_nat_gateway'", (scan,), lambda failed:any(r['check']=='regional:aws_vpc_nat_gateway:root-scope-present' for r in failed))
    altered('incomplete-child-scope', "UPDATE scope_checkpoints SET complete=0 WHERE scan_id=? AND collector='aws_elbv2_listener'", (scan,), lambda failed:any(':scope-complete' in r['check'] for r in failed))
    altered('resource-property-byte-cap', "UPDATE resource_observations SET resource_json=json_set(resource_json,'$.properties.padding',?) WHERE scan_id=? AND resource_id=?", ('x'*(assert_regional.MAX_PROPERTY_BYTES+1),scan,instance), lambda failed:any(r['check']=='regional:input-validation' for r in failed))

    wrong=copy.deepcopy(fixtures)
    wrong['ecs']['clusters'][0]['service_task_definitions']={service:wrong_definition}
    check=assert_regional.check(db,scan,wrong)
    verify('creation-reference-map-is-authoritative',any(not r['passed'] and r['check']=='regional:'+service+':task_definition' for r in check))

    # Mutate only the in-memory copy, with read-back assertions unchanged.
    db.execute('SAVEPOINT row_cap')
    try:
        columns=[row[1] for row in db.execute('PRAGMA table_info(resource_observations)')]
        row=list(db.execute('SELECT * FROM resource_observations WHERE scan_id=? AND resource_id=? LIMIT 1',(scan,instance)).fetchone())
        identity_column=columns.index('resource_id')
        statement='INSERT INTO resource_observations ('+','.join(columns)+') VALUES ('+','.join('?' for _ in columns)+')'
        for index in range(assert_regional.MAX_RESOURCES):
            row[identity_column]='bounded-fixture-row-'+str(index)
            db.execute(statement,row)
        check=assert_regional.check(db,scan,fixtures)
        verify('resource-row-cap',any(not r['passed'] and r['check']=='regional:input-validation' and 'row cap' in r.get('detail',{}).get('error','') for r in check))
    finally:
        db.execute('ROLLBACK TO row_cap')
        db.execute('RELEASE row_cap')
    verify('read-only-checker-and-test-restoration',assert_regional.check(db,scan,fixtures)==baseline)
    (base/'regional-assertion-self-test.json').write_text(json.dumps({'cases':results,'all_passed':all(r['passed'] for r in results)},indent=2,sort_keys=True)+'\n')
    print(json.dumps({'baseline_checks':len(baseline),'cases':len(results),'failed':[r for r in results if not r['passed']]},indent=2))
    return 0 if all(r['passed'] for r in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
