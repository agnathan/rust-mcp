# Step tags

Each chapter ends with an annotated git tag `step-NN` on the merge commit, after that chapter's pull request is merged to `main`. The tag is not part of the pull request. Do not push it from the chapter branch.

| Tag | Chapter | When |
|---|---|---|
| `step-01` | Chapter 1, Speaking JSON-RPC | After the chapter 1 pull request merges. Not created and not pushed yet. |

The command, run on `main` after the merge, and not before:

```bash
git checkout main
git pull
git tag -a step-01 -m "Chapter 1: Speaking JSON-RPC"
git push origin step-01
```
