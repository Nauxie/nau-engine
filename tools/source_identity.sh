#!/usr/bin/env bash

nau_sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  else
    sha256sum | awk '{print $1}'
  fi
}

nau_source_commit() {
  git rev-parse HEAD
}

nau_source_state() {
  if [[ -z "$(git status --porcelain --untracked-files=all)" ]]; then
    printf 'clean\n'
  else
    printf 'dirty\n'
  fi
}

nau_source_fingerprint() {
  {
    printf 'commit:%s\n' "$(nau_source_commit)"
    git diff --binary HEAD --
    while IFS= read -r file_path; do
      [[ -n "${file_path}" ]] || continue
      printf 'untracked:%s:' "${file_path}"
      nau_sha256 < "${file_path}"
    done < <(git ls-files --others --exclude-standard | LC_ALL=C sort)
  } | nau_sha256
}
