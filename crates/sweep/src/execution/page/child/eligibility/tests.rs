
    use super::*;
    use inventory_core::{
        inventory::{AccountId, OperationId, ServiceId},
        record::Key,
    };
    use std::collections::BTreeMap;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn scope() -> Result<ScopeKey, Box<dyn std::error::Error>> {
        Ok(ScopeKey::new(
            AccountId::parse("123456789012")?,
            Region::parse("us-east-1")?,
            ServiceId::parse("sns")?,
            OperationId::parse("ListSubscriptions")?,
            None,
        )?)
    }
    fn row(text: &str) -> Result<Record, Box<dyn std::error::Error>> {
        Ok(Record::encode(&Value::Map(BTreeMap::from([(
            Key::Field(FieldId::new(1)?),
            Value::Text(text.to_owned()),
        )])))?)
    }
    #[test]
    fn given_confirmed_subscription_when_selected_then_exact_arn_is_bound() -> TestResult {
        let text =
            "arn:aws:sns:us-east-1:123456789012:topic.fifo:0fbbf013-2c09-4c13-b31a-62857ec41b47";
        assert_eq!(
            value(
                &row(text)?,
                &[FieldId::new(1)?],
                "sns",
                ArnResourceKind::SnsSubscription,
                &scope()?
            )?,
            Some(Value::Text(text.to_owned()))
        );
        Ok(())
    }
    #[test]
    fn given_only_documented_sentinels_when_selected_then_detail_is_not_scheduled() -> TestResult {
        for text in ["PendingConfirmation", "Deleted"] {
            assert_eq!(
                value(
                    &row(text)?,
                    &[FieldId::new(1)?],
                    "sns",
                    ArnResourceKind::SnsSubscription,
                    &scope()?
                )?,
                None
            );
        }
        Ok(())
    }
    #[test]
    fn given_malformed_or_wrong_owner_arns_when_selected_then_failure_is_explicit() -> TestResult {
        for text in [
            "",
            "arn:",
            "pendingconfirmation",
            "arn:aws:sns:us-east-1:123456789012:topic",
            "arn:aws:sns:us-east-1:123456789012:topic:bad/id",
            "arn:aws:sns:us-east-1:999999999999:topic:id",
            "arn:aws:sns:us-west-2:123456789012:topic:id",
            "arn:aws-cn:sns:us-east-1:123456789012:topic:id",
            "arn:aws:sqs:us-east-1:123456789012:topic:id",
        ] {
            assert!(
                matches!(
                    value(
                        &row(text)?,
                        &[FieldId::new(1)?],
                        "sns",
                        ArnResourceKind::SnsSubscription,
                        &scope()?
                    ),
                    Err(SweepError::Operation(
                        "reviewed parent ARN is invalid or outside its trusted scope"
                    ))
                ),
                "{text}"
            );
        }
        let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
        assert!(matches!(
            value(
                &empty,
                &[FieldId::new(1)?],
                "sns",
                ArnResourceKind::SnsSubscription,
                &scope()?
            ),
            Err(SweepError::Operation(
                "reviewed parent ARN is missing or not text"
            ))
        ));
        Ok(())
    }
