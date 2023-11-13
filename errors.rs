use ink::prelude::string::String;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP34Error {
    /// Custom error type for cases if writer of traits added own restrictions
    Custom(String),
    /// Returned if owner approves self
    SelfApprove,
    /// Returned if the caller doesn't have allowance for transferring.
    NotApproved,
    /// Returned if the owner already own the token.
    TokenExists,
    /// Returned if the token doesn't exist
    TokenNotExists,
    /// Returned if reached max supply
    ReachedMaxSupply,
    /// Returned if safe transfer check fails
    SafeTransferCheckFailed(String),
    /// Returned if finding token index not in owners collection
    OutOfBoundsIndex,
    /// Returned if trying to call approve when operator has all approved
    NotAllowedToApprove
}