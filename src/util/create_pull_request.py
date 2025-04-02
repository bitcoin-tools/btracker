'''This script creates a new GitHub PR targeting master.```

# TODO: add auto-merge

from github import Github
import os

g = Github(os.getenv("GITHUB_TOKEN"))
repo = g.get_repo("bitcoin-tools/btracker")  # Replace with your username/repo

branch_name = "update-price"
repo.create_git_ref(ref=f"refs/heads/{branch_name}", sha=repo.get_branch("master").commit.sha)

os.system('git add .')
os.system('git commit -m "Update price data"')
os.system(f"git push origin {branch_name}")

pr = repo.create_pull(
    title="chore(data): update prices",
    body="Auto-generated PR to update the price data.",
    head=branch_name,
    base="master",
)

print(f"Pull request created: {pr.html_url}")
