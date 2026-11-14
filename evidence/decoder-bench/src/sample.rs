use crate::error::{Result, syntax};
use std::fmt::Write;
pub struct Sample {
    pub json: Vec<u8>,
    pub xml: Vec<u8>,
    pub items: usize,
}
pub fn generate(items: usize, padding: usize) -> Result<Sample> {
    let mut json = String::from("{\"Items\":[");
    let mut xml = String::from(
        "<DescribeInstancesResponse xmlns=\"http://ec2.amazonaws.com/doc/2016-11-15/\"><requestId>fixed-request</requestId><reservationSet><item><reservationId>r-fixed</reservationId><instancesSet>",
    );
    (0..items).try_for_each(|i| {
        if i != 0 {
            json.push(',');
        }
        item(&mut json, &mut xml, i, padding)
    })?;
    json.push_str("],\"NextToken\":null,\"Unknown\":{\"future\":[1,true,null,\"safe\"]}}");
    xml.push_str("</instancesSet></item></reservationSet><nextToken></nextToken><Unknown><future>safe</future></Unknown></DescribeInstancesResponse>");
    Ok(Sample {
        json: json.into_bytes(),
        xml: xml.into_bytes(),
        items,
    })
}
fn item(json: &mut String, xml: &mut String, i: usize, padding: usize) -> Result<()> {
    let state = (0..padding).fold(
        0x5eed_u64.wrapping_add(u64::try_from(i).map_err(syntax)?),
        |s, _| s.wrapping_mul(6364136223846793005).wrapping_add(1),
    );
    let pad: String = (0..padding)
        .scan(state, |s, _| {
            *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            let byte = match u8::try_from(*s % 26) {
                Ok(v) => v,
                Err(_) => return None,
            };
            Some(char::from(b'a'.saturating_add(byte)))
        })
        .collect();
    write!(json, "{{\"Ratio\":0.125,\"InstanceId\":\"i-{i:017x}\",\"State\":{{\"Name\":\"running\",\"Code\":16}},\"SecurityGroups\":[{{\"GroupId\":\"sg-1\",\"GroupName\":\"web\"}},{{\"GroupId\":\"sg-2\",\"GroupName\":\"db\"}}],\"Tags\":{{\"owner\":\"Zoë 東京\",\"env\":\"prod\"}},\"Description\":\"quote\\\" slash\\\\ line\\n snowman\\u2603 rocket\\ud83d\\ude80 {pad}\",\"CpuCount\":8,\"LargeLong\":9223372036854775806,\"Enabled\":true,\"Optional\":null,\"LaunchTime\":1735689600.125,\"UserData\":\"AAEC/f7/\",\"Unknown\":{{\"ignore\":[false,{{\"x\":\"é\"}}]}}}}").map_err(syntax)?;
    write!(xml, "<item><ratio>0.125</ratio><instanceId>i-{i:017x}</instanceId><instanceState><name>running</name><code>16</code></instanceState><groupSet><item><groupId>sg-1</groupId><groupName>web</groupName></item><item><groupId>sg-2</groupId><groupName>db</groupName></item></groupSet><tagSet><item><key>owner</key><value>Zoë 東京</value></item><item><key>env</key><value>prod</value></item></tagSet><description>quote&quot; slash\\ line&#10; snowman☃ rocket🚀 {pad}</description><cpuCount>8</cpuCount><largeLong>9223372036854775806</largeLong><enabled>true</enabled><optional/><launchTime>2025-01-01T00:00:00.125Z</launchTime><userData>AAEC/f7/</userData><Unknown><ignore>é</ignore></Unknown></item>").map_err(syntax)
}
