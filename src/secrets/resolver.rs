use anyhow::Result;
use secrecy::SecretString;

/// Read-only resolver for external secret managers.
///
/// Each implementation handles a specific URI scheme (e.g. `op://`, `vault://`).
/// The `resolve` method receives the full URI reference and returns the secret value.
pub trait SecretResolver: Send + Sync {
    /// The URI scheme this resolver handles (e.g. "op", "vault", "aws").
    fn scheme(&self) -> &str;

    /// Resolve a secret reference to its value.
    fn resolve(&self, reference: &str) -> Result<SecretString>;
}
