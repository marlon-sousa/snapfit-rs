//! Error: every problem found in one expansion, gathered so that none hides another.

use super::DomainError;

#[cfg(test)]
mod tests;

/// Every problem found in one expansion, never empty, so that the user fixes
/// them all in one pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainErrors {
    first: DomainError,
    rest: Vec<DomainError>,
}

impl DomainErrors {
    /// `Ok` when `errors` is empty.
    pub(crate) fn check(errors: Vec<DomainError>) -> Result<(), Self> {
        let mut errors = errors.into_iter();
        errors.next().map_or(Ok(()), |first| {
            Err(Self {
                first,
                rest: errors.collect(),
            })
        })
    }

    /// Every value when every result is `Ok`, otherwise every error of every result.
    pub(crate) fn collect<T>(
        results: impl IntoIterator<Item = Result<T, Self>>,
    ) -> Result<Vec<T>, Self> {
        let (values, failures): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        let errors = failures
            .into_iter()
            .filter_map(Result::err)
            .flat_map(Self::into_iter)
            .collect();
        Self::check(errors)?;
        Ok(values.into_iter().filter_map(Result::ok).collect())
    }

    /// Both values, or the errors of either or both.
    pub(crate) fn both<A, B>(
        first: Result<A, Self>,
        second: Result<B, Self>,
    ) -> Result<(A, B), Self> {
        match (first, second) {
            (Ok(first), Ok(second)) => Ok((first, second)),
            (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
            (Err(mut errors), Err(more)) => {
                errors.rest.extend(more);
                Err(errors)
            }
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &DomainError> {
        std::iter::once(&self.first).chain(&self.rest)
    }
}

impl IntoIterator for DomainErrors {
    type Item = DomainError;
    type IntoIter = std::iter::Chain<std::iter::Once<DomainError>, std::vec::IntoIter<DomainError>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.first).chain(self.rest)
    }
}

impl From<DomainError> for DomainErrors {
    fn from(first: DomainError) -> Self {
        Self {
            first,
            rest: Vec::new(),
        }
    }
}
