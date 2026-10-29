# Desktop Feedback

## ADDED Requirements

### Requirement: Durable Notification History

The desktop SHALL deliver interface feedback through toasts and retain notification time, source and detail until the user clears history.

#### Scenario: Error Feedback

- **WHEN** an interface action fails
- **THEN** its error appears in a sticky toast and notification history without adding inline error text beside its control
- **AND** the operation retains its retry control and unsaved draft where applicable

#### Scenario: Toast Dismissal

- **WHEN** the user dismisses a toast or informational feedback closes automatically
- **THEN** the corresponding history remains available in Notifications
- **AND** updating a visible toast does not discard earlier recorded messages

#### Scenario: History Navigation And Clear

- **WHEN** the user browses history larger than one page
- **THEN** bounded pages expose every retained record, with an unread badge in navigation
- **AND** opening Notifications marks its messages read
- **WHEN** the user clears notification history
- **THEN** its saved records and unread count are removed

#### Scenario: Storage Failure

- **WHEN** history cannot be opened or written
- **THEN** the failure is surfaced without clearing existing records
- **AND** a later attempt may reopen storage without resetting its data
