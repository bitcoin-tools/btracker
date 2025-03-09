from github import Github
import os

# Authenticate with GitHub
g = Github(os.getenv("GITHUB_TOKEN"))

# Get repository
repo = g.get_repo("bitcoin-tools/btracker")  # Replace with your username/repo

# Create a new branch for the update
branch_name = "update-price"
repo.create_git_ref(ref=f"refs/heads/{branch_name}", sha=repo.get_branch("main").commit.sha)

# Commit changes and push to the new branch
os.system(f"git add .")
os.system(f'git commit -m "Update price data"')
os.system(f"git push origin {branch_name}")

# Create Pull Request
pr = repo.create_pull(
    title="Update Price Data",
    body="Auto-generated PR from This PR updates the price data.",
    head=branch_name,
    base="main",
)

print(f"Pull request created: {pr.html_url}")
