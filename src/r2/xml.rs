use serde::Deserialize;

use crate::error::*;

#[derive(Deserialize)]
struct ListBucketResult {
    #[serde(default, rename = "Contents")]
    contents: Vec<Contents>,

    #[serde(default, rename = "IsTruncated")]
    is_truncated: bool,

    #[serde(rename = "NextContinuationToken")]
    next_continuation_token: Option<String>,
}

#[derive(Deserialize)]
struct Contents {
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    #[serde(rename = "Message")]
    message: Option<String>,
}

pub fn parse_list(xml: &str) -> RsyncResult<(Vec<String>, Option<String>)> {
    let page: ListBucketResult = serde_xml_rs::from_str(xml)?;

    let keys = page
        .contents
        .into_iter()
        .map(|content| content.key)
        .collect();

    let token = if page.is_truncated {
        Some(
            page.next_continuation_token
                .ok_or_else(|| Error::R2("list response has no continuation token".into()))?,
        )
    } else {
        None
    };

    Ok((keys, token))
}

pub fn error_message(xml: &str) -> String {
    serde_xml_rs::from_str::<ErrorResponse>(xml)
        .ok()
        .and_then(|response| response.message)
        .unwrap_or_else(|| xml.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_page() {
        let xml = r#"
            <ListBucketResult>
                <IsTruncated>false</IsTruncated>
                <KeyCount>2</KeyCount>
                <Contents><Key>a.txt</Key></Contents>
                <Contents>
                    <Key>dir/b.txt</Key>
                    <LastModified>2024-01-01T00:00:00.000Z</LastModified>
                    <Size>5</Size>
                </Contents>
            </ListBucketResult>"#;

        let (keys, token) = parse_list(xml).unwrap();

        assert_eq!(keys, vec!["a.txt".to_string(), "dir/b.txt".to_string()]);
        assert_eq!(token, None);
    }

    #[test]
    fn truncated_page() {
        let xml = r#"
            <ListBucketResult>
                <IsTruncated>true</IsTruncated>
                <NextContinuationToken>next-token/abc==</NextContinuationToken>
                <Contents><Key>a.txt</Key></Contents>
            </ListBucketResult>"#;

        let (keys, token) = parse_list(xml).unwrap();

        assert_eq!(keys, vec!["a.txt".to_string()]);
        assert_eq!(token, Some("next-token/abc==".to_string()));
    }

    #[test]
    fn empty_page() {
        let (keys, token) = parse_list("<ListBucketResult/>").unwrap();

        assert!(keys.is_empty());
        assert_eq!(token, None);
    }

    #[test]
    fn default_page() {
        let xml = r#"
            <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
                <IsTruncated>false</IsTruncated>
                <Contents><Key>a.txt</Key></Contents>
            </ListBucketResult>"#;

        let (keys, token) = parse_list(xml).unwrap();

        assert_eq!(keys, vec!["a.txt".to_string()]);
        assert_eq!(token, None);
    }

    #[test]
    fn error_message_body() {
        let xml = r#"
            <Error>
                <Code>NoSuchBucket</Code>
                <Message>The specified bucket does not exist</Message>
            </Error>"#;

        assert_eq!(error_message(xml), "The specified bucket does not exist");
    }

    #[test]
    fn truncated_page_without_token() {
        let xml = r#"
            <ListBucketResult>
                <IsTruncated>true</IsTruncated>
            </ListBucketResult>"#;

        assert!(matches!(parse_list(xml), Err(Error::R2(_))));
    }
}
