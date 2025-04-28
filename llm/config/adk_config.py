#!/usr/bin/env python3
"""
ADK Configuration for Patina Language Development
"""

import os
from functools import partial
from typing import Dict, Optional

from google import adk


# Configuration functions for the Patina ADK setup
def get_project_config(
    project_id: Optional[str] = None, location: Optional[str] = None
) -> tuple:
    """Get project configuration parameters.

    Args:
        project_id: GCP project ID (defaults to env var GOOGLE_CLOUD_PROJECT)
        location: GCP location (defaults to env var GOOGLE_CLOUD_LOCATION or 'us-central1')

    Returns:
        tuple: (project_id, location, adk_instance)
    """
    resolved_project_id = project_id or os.environ.get("GOOGLE_CLOUD_PROJECT")
    if not resolved_project_id:
        raise ValueError(
            "Project ID must be provided or set in GOOGLE_CLOUD_PROJECT env var"
        )

    resolved_location = location or os.environ.get(
        "GOOGLE_CLOUD_LOCATION", "us-central1"
    )

    # Initialize ADK
    adk_instance = adk.ADK()

    return resolved_project_id, resolved_location, adk_instance


def create_agent_config(
    project_id: str,
    location: str,
    agent_name: str,
    prompt_file: str,
    model: str = "gemini-2.5-pro",
) -> adk.AgentConfig:
    """Create configuration for an agent.

    Args:
        project_id: GCP project ID
        location: GCP location
        agent_name: Name of the agent
        prompt_file: Path to the prompt file for this agent
        model: LLM model to use

    Returns:
        AgentConfig: Configuration for the agent
    """
    with open(prompt_file, "r") as f:
        prompt = f.read()

    return adk.AgentConfig(
        name=agent_name,
        model=model,
        system_prompt=prompt,
        vertex=adk.VertexConfig(project=project_id, location=location),
    )


def get_prompt_path(agent_name: str) -> str:
    """Get the path to a prompt file for an agent.

    Args:
        agent_name: Name of the agent

    Returns:
        str: Path to the prompt file
    """
    return os.path.join(os.path.dirname(__file__), f"../prompts/{agent_name}.md")


def initialize_agents(project_id: str, location: str) -> Dict[str, adk.AgentConfig]:
    """Initialize all Patina agents.

    Args:
        project_id: GCP project ID
        location: GCP location

    Returns:
        dict: Dictionary mapping agent names to their configurations
    """
    # Create a partially applied function with project_id and location already set
    create_config = partial(create_agent_config, project_id, location)

    # Define all agent names
    agent_names = [
        "coordinator",
        "syntax",
        "semantics",
        "memory_safety",
        "ir",
        "testing",
        "qa_validation",
        "infrastructure",
    ]

    # Map each agent name to its configuration
    return {name: create_config(name, get_prompt_path(name)) for name in agent_names}
