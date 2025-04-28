#!/usr/bin/env python3
"""
Repository tools for Patina Language Development agents using functional programming
"""

import logging
import os
import subprocess
from typing import Any, Dict, List, Optional

from google import adk

logger = logging.getLogger("patina-repo-tools")


def get_repo_config(repo_path: str) -> Dict[str, str]:
    """Get repository configuration parameters.

    Args:
        repo_path: Path to the repository

    Returns:
        dict: Repository configuration parameters
    """
    return {"repo_path": repo_path}


async def read_repo_file(repo_config: Dict[str, str], file_path: str) -> str:
    """Read a file from the repository.

    Args:
        repo_config: Repository configuration
        file_path: Path to the file, relative to repo root

    Returns:
        str: File content
    """
    abs_path = os.path.join(repo_config["repo_path"], file_path)
    logger.info(f"Reading file: {abs_path}")

    try:
        with open(abs_path, "r") as f:
            return f.read()
    except Exception as e:
        logger.error(f"Error reading file {abs_path}: {e}")
        return f"Error: {str(e)}"


def read_file_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for reading files from the repository.

    Args:
        repo_config: Repository configuration

    Returns:
        Tool: The read file tool
    """

    async def tool_function(file_path: str) -> str:
        return await read_repo_file(repo_config, file_path)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="read_file",
            description="Read a file from the repository",
            parameters=[
                adk.ToolParameter(
                    name="file_path",
                    description="Path to the file, relative to repo root",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                )
            ],
        ),
        function=tool_function,
    )


async def write_repo_file(
    repo_config: Dict[str, str], file_path: str, content: str, create_dirs: bool = True
) -> str:
    """Write content to a file in the repository.

    Args:
        repo_config: Repository configuration
        file_path: Path to the file, relative to repo root
        content: Content to write
        create_dirs: Whether to create parent directories

    Returns:
        str: Success message or error
    """
    abs_path = os.path.join(repo_config["repo_path"], file_path)
    logger.info(f"Writing to file: {abs_path}")

    try:
        if create_dirs:
            os.makedirs(os.path.dirname(abs_path), exist_ok=True)

        with open(abs_path, "w") as f:
            f.write(content)

        return f"Successfully wrote to {file_path}"
    except Exception as e:
        logger.error(f"Error writing to file {abs_path}: {e}")
        return f"Error: {str(e)}"


def write_file_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for writing files to the repository.

    Args:
        repo_config: Repository configuration

    Returns:
        Tool: The write file tool
    """

    async def tool_function(
        file_path: str, content: str, create_dirs: bool = True
    ) -> str:
        return await write_repo_file(repo_config, file_path, content, create_dirs)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="write_file",
            description="Write content to a file in the repository",
            parameters=[
                adk.ToolParameter(
                    name="file_path",
                    description="Path to the file, relative to repo root",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="content",
                    description="Content to write to the file",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="create_dirs",
                    description="Whether to create parent directories",
                    type=adk.ToolParameterType.BOOLEAN,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


async def run_repo_command(
    repo_config: Dict[str, str], command: str, cwd: Optional[str] = None
) -> Dict[str, Any]:
    """Run a command in the repository.

    Args:
        repo_config: Repository configuration
        command: Command to run
        cwd: Working directory (defaults to repo root)

    Returns:
        dict: Command output with stdout, stderr, and return code
    """
    working_dir = (
        os.path.join(repo_config["repo_path"], cwd) if cwd else repo_config["repo_path"]
    )
    logger.info(f"Running command: {command} in {working_dir}")

    try:
        result = subprocess.run(
            command, shell=True, cwd=working_dir, capture_output=True, text=True
        )

        return {
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode,
        }
    except Exception as e:
        logger.error(f"Error running command {command}: {e}")
        return {"stdout": "", "stderr": str(e), "returncode": -1}


def run_command_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for running commands in the repository.

    Args:
        repo_config: Repository configuration

    Returns:
        Tool: The run command tool
    """

    async def tool_function(command: str, cwd: Optional[str] = None) -> Dict[str, Any]:
        return await run_repo_command(repo_config, command, cwd)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="run_command",
            description="Run a command in the repository",
            parameters=[
                adk.ToolParameter(
                    name="command",
                    description="Command to run",
                    type=adk.ToolParameterType.STRING,
                    required=True,
                ),
                adk.ToolParameter(
                    name="cwd",
                    description="Working directory relative to repo root",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


async def run_repo_tests(
    repo_config: Dict[str, str],
    package: Optional[str] = None,
    update_snapshots: bool = False,
) -> Dict[str, Any]:
    """Run tests in the repository.

    Args:
        repo_config: Repository configuration
        package: Specific package to test (defaults to all)
        update_snapshots: Whether to update snapshots

    Returns:
        dict: Test results with stdout, stderr, and return code
    """
    cmd = "go test"

    if package:
        cmd += f" ./{package}/..."
    else:
        cmd += " ./..."

    if update_snapshots:
        cmd = f"UPDATE_SNAPS=true {cmd}"

    logger.info(f"Running tests: {cmd}")

    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=repo_config["repo_path"],
            capture_output=True,
            text=True,
        )

        return {
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode,
            "success": result.returncode == 0,
        }
    except Exception as e:
        logger.error(f"Error running tests: {e}")
        return {
            "stdout": "",
            "stderr": str(e),
            "returncode": -1,
            "success": False,
        }


def run_tests_tool(repo_config: Dict[str, str]) -> adk.Tool:
    """Create a tool for running tests in the repository.

    Args:
        repo_config: Repository configuration

    Returns:
        Tool: The run tests tool
    """

    async def tool_function(
        package: Optional[str] = None, update_snapshots: bool = False
    ) -> Dict[str, Any]:
        return await run_repo_tests(repo_config, package, update_snapshots)

    return adk.Tool(
        spec=adk.ToolSpec(
            name="run_tests",
            description="Run tests in the repository",
            parameters=[
                adk.ToolParameter(
                    name="package",
                    description="Specific package to test",
                    type=adk.ToolParameterType.STRING,
                    required=False,
                ),
                adk.ToolParameter(
                    name="update_snapshots",
                    description="Whether to update snapshots",
                    type=adk.ToolParameterType.BOOLEAN,
                    required=False,
                ),
            ],
        ),
        function=tool_function,
    )


def get_repo_tools(repo_path: str) -> List[adk.Tool]:
    """Get all repository tools.

    Args:
        repo_path: Path to the repository

    Returns:
        list: List of Tool instances
    """
    repo_config = get_repo_config(repo_path)

    return [
        read_file_tool(repo_config),
        write_file_tool(repo_config),
        run_command_tool(repo_config),
        run_tests_tool(repo_config),
    ]
