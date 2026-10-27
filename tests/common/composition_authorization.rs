//! Authorized completion edits for negative IME fixtures, never runtime code.
pub fn accepted_edit(from: &str, to: &str) -> lay::text_edit::AuthorizedEdit {
    let action = lay::text_edit::plan_ime_candidate_accept_edit("fixture", 900, from, to);
    lay::text_edit::authorize_backend_edit(lay::text_edit::TextEditBackend::Ime, action)
        .into_authorized()
        .expect("fixture authorized edit")
}
