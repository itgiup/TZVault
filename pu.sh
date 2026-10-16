git add .
git commit -m "feat: add 'Create a new vault file' to the unlock screen" -m "- Add a 'create' mode to cmd_set_db_path: creates an empty vault at a chosen path, then routes to Setup. Refuses to overwrite non-empty files (ERR_FILE_ALREADY_EXISTS).
- Setup screen: add 'Use a different vault file' as an escape hatch and show the target path.
- Storage::switch_path now swaps only after the new file opens and initializes, so a bad file no longer leaves the app pointed at it.
- Drop needless borrows flagged by clippy.
- Update README (EN/VI)."
git push
# /home/<user>/.local/share/com.u.tauri-app

# git checkout main
# git merge master
# git push
# git checkout master