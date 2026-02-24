#![cfg(feature = "secrets-aws")]

use earl::secrets::resolver::SecretResolver;
use earl::secrets::resolvers::aws::AwsResolver;

#[test]
fn aws_resolver_scheme() {
    let resolver = AwsResolver::new();
    assert_eq!(resolver.scheme(), "aws");
}

#[test]
fn aws_resolver_rejects_empty_name() {
    let resolver = AwsResolver::new();
    let err = resolver.resolve("aws://").unwrap_err();
    assert!(err.to_string().contains("invalid"), "got: {}", err);
}
