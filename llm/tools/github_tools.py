#!/usr/bin/env python3
"""
GitHub tools for Patina Language Development agents using functional programming
"""

import logging
import os
from typing import Any, Callable, Dict, List, Optional

from google import adk

logger = logging.getLogger("patina-github-tools")


def get_github_config(
    repo_owner: str, repo_name: str, token: Optional[str] = None
) -> Dict[str, str]:
    """Get GitHub configuration parameters.

    Args:
        repo_owner: GitHub repository owner
        repo_name: GitHub repository name
        token: GitHub token (defaults to env var GITHUB_TOKEN)

    Returns:
        dict: GitHub configuration parameters
    """
    resolved_token = token or os.environ.get("GITHUB_TOKEN")
    if not resolved_token:
        logger.warning("GitHub token not provided. Some operations may fail.")

    return {"repo_owner": repo_owner, "repo_name": repo_name, "token": resolved_token}


async def create_github_issue(
    repo_config: Dict[str, str],
    title: str,
    body: str,
    labels: Optional[List[str]] = None,
) -> Dict[str, Any]:
    """Create a GitHub issue.

    Args:
        repo_config: GitHub repository configuration
        title: Issue title
        body: Issue body
        labels: Optional list of labels

    Returns:
        dict: Issue data
    """
    # Implementation would use GitHub API
    logger.info(f"Creating issue: {title}")
    return {"title": title, "body": body, "labels": labels, "status": "created"}


def create_issue_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for creating GitHub issues.

    Args:
        repo_config: GitHub repository configuration

    Returns:
        Tool: The create issue tool
    """

    # Create a function that captures the repo_config
    async def tool_function(
        title: str, body: str, labels: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        return await create_github_issue(repo_config, title, body, labels)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="create_github_issue",
            description="Create a new issue in the GitHub repository",
            parameters=[
                adk.ToolParameter(
                    name="title",
                    description="Issue title",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="body",
                    description="Issue body",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="labels",
                    description="Issue labels",
                    type=adk.ToolParameterType.ARRAY,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


async def list_github_issues(
    repo_config: Dict[str, str], state: str = "open", labels: Optional[List[str]] = None
) -> List[Dict[str, Any]]:
    """List GitHub issues.

    Args:
        repo_config: GitHub repository configuration
        state: Issue state (open, closed, all)
        labels: Optional list of labels to filter by

    Returns:
        list: List of issues
    """
    # Implementation would use GitHub API
    logger.info(f"Listing issues with state: {state}")
    return [{"title": "Example issue", "state": state}]


def list_issues_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for listing GitHub issues.

    Args:
        repo_config: GitHub repository configuration

    Returns:
        Tool: The list issues tool
    """

    async def tool_function(
        state: str = "open", labels: Optional[List[str]] = None
    ) -> List[Dict[str, Any]]:
        return await list_github_issues(repo_config, state, labels)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="list_github_issues",
            description="List issues in the GitHub repository",
            parameters=[
                adk.ToolParameter(
                    name="state",
                    description="Issue state (open, closed, all)",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                ),
                adk.ToolParameter(
                    name="labels",
                    description="Filter by labels",
                    type=adk.ToolParameterType.ARRAY,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


async def create_github_pull_request(
    repo_config: Dict[str, str], title: str, body: str, head: str, base: str = "main"
) -> Dict[str, Any]:
    """Create a GitHub pull request.

    Args:
        repo_config: GitHub repository configuration
        title: PR title
        body: PR body
        head: Head branch
        base: Base branch

    Returns:
        dict: Pull request data
    """
    # Implementation would use GitHub API
    logger.info(f"Creating PR: {title} from {head} to {base}")
    return {
        "title": title,
        "body": body,
        "head": head,
        "base": base,
        "status": "created",
    }


def create_pull_request_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for creating GitHub pull requests.

    Args:
        repo_config: GitHub repository configuration

    Returns:
        Tool: The create pull request tool
    """

    async def tool_function(
        title: str, body: str, head: str, base: str = "main"
    ) -> Dict[str, Any]:
        return await create_github_pull_request(repo_config, title, body, head, base)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="create_github_pull_request",
            description="Create a new pull request in the GitHub repository",
            parameters=[
                adk.ToolParameter(
                    name="title",
                    description="Pull request title",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="body",
                    description="Pull request body",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="head",
                    description="Head branch",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="base",
                    description="Base branch",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


async def list_github_pull_requests(
    repo_config: Dict[str, str], state: str = "open"
) -> List[Dict[str, Any]]:
    """List GitHub pull requests.

    Args:
        repo_config: GitHub repository configuration
        state: PR state (open, closed, all)

    Returns:
        list: List of pull requests
    """
    # Implementation would use GitHub API
    logger.info(f"Listing PRs with state: {state}")
    return [{"title": "Example PR", "state": state}]


def list_pull_requests_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for listing GitHub pull requests.

    Args:
        repo_config: GitHub repository configuration

    Returns:
        Tool: The list pull requests tool
    """

    async def tool_function(state: str = "open") -> List[Dict[str, Any]]:
        return await list_github_pull_requests(repo_config, state)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="list_github_pull_requests",
            description="List pull requests in the GitHub repository",
            parameters=[
                adk.ToolParameter(
                    name="state",
                    description="Pull request state (open, closed, all)",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                )
            ],
        ),
        function=tool_function,
    )


async def get_github_repository_content(
    repo_config: Dict[str, str], path: str, ref: str = "main"
) -> Dict[str, Any]:
    """Get repository content.

    Args:
        repo_config: GitHub repository configuration
        path: Path to the content
        ref: Git reference

    Returns:
        dict: Content data
    """
    # Implementation would use GitHub API
    logger.info(f"Getting repo content: {path} at {ref}")
    return {"path": path, "ref": ref, "content": "Example content"}


def get_repository_content_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for getting repository content.

    Args:
        repo_config: GitHub repository configuration

    Returns:
        Tool: The get repository content tool
    """

    async def tool_function(path: str, ref: str = "main") -> Dict[str, Any]:
        return await get_github_repository_content(repo_config, path, ref)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="get_github_repository_content",
            description="Get content from the GitHub repository",
            parameters=[
                adk.ToolParameter(
                    name="path",
                    description="Path to the content",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="ref",
                    description="Git reference",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


def get_github_tools(
    repo_owner: str, repo_name: str, token: Optional[str] = None
) -> List[adk.Tool]:
    """Get all GitHub tools.

    Args:
        repo_owner: GitHub repository owner
        repo_name: GitHub repository name
        token: GitHub token (defaults to env var GITHUB_TOKEN)

    Returns:
        list: List of Tool instances
    """
    repo_config = get_github_config(repo_owner, repo_name, token)

    return [
        create_issue_tool(repo_config),
        list_issues_tool(repo_config),
        create_pull_request_tool(repo_config),
        list_pull_requests_tool(repo_config),
        get_repository_content_tool(repo_config),
    ]
