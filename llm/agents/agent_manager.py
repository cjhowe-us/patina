#!/usr/bin/env python3
"""
Agent Manager for Patina Language Development using functional programming
"""

import logging
from typing import Any, Dict, List, Optional

from google import adk

from ..config.adk_config import get_project_config, initialize_agents

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("patina-agent-manager")


def setup_agent_environment(
    project_id: Optional[str] = None, location: Optional[str] = None
) -> tuple:
    """Set up the agent environment and configurations.

    Args:
        project_id: GCP project ID (defaults to env var)
        location: GCP location (defaults to env var or 'us-central1')

    Returns:
        tuple: (project_id, location, agent_configs)
    """
    project_id, location, _ = get_project_config(project_id, location)
    agent_configs = initialize_agents(project_id, location)

    return project_id, location, agent_configs


def initialize_agent(
    agent_configs: Dict[str, adk.AgentConfig],
    agents: Dict[str, adk.Agent],
    agent_name: str,
) -> adk.Agent:
    """Initialize a specific agent.

    Args:
        agent_configs: Dictionary of agent configurations
        agents: Dictionary of initialized agents
        agent_name: Name of the agent to initialize

    Returns:
        Agent: The initialized agent

    Raises:
        ValueError: If the agent name is not recognized
    """
    if agent_name not in agent_configs:
        raise ValueError(f"Unknown agent: {agent_name}")

    if agent_name not in agents:
        logger.info(f"Initializing agent: {agent_name}")
        agents[agent_name] = adk.Agent(agent_configs[agent_name])

    return agents[agent_name]


def initialize_all_agents(
    agent_configs: Dict[str, adk.AgentConfig]
) -> Dict[str, adk.Agent]:
    """Initialize all agents.

    Args:
        agent_configs: Dictionary of agent configurations

    Returns:
        dict: Dictionary mapping agent names to initialized agents
    """
    agents = {}

    for agent_name in agent_configs:
        initialize_agent(agent_configs, agents, agent_name)

    return agents


def get_agent(agents: Dict[str, adk.Agent], agent_name: str) -> adk.Agent:
    """Get an initialized agent by name.

    Args:
        agents: Dictionary of initialized agents
        agent_name: Name of the agent

    Returns:
        Agent: The initialized agent

    Raises:
        ValueError: If the agent is not initialized
    """
    if agent_name not in agents:
        raise ValueError(
            f"Agent {agent_name} is not initialized. Call initialize_agent() first."
        )

    return agents[agent_name]


def run_agent(
    agents: Dict[str, adk.Agent],
    agent_name: str,
    message: str,
    tools: Optional[List[adk.Tool]] = None,
) -> Any:
    """Run an agent with a message and optional tools.

    Args:
        agents: Dictionary of initialized agents
        agent_name: Name of the agent to run
        message: Message to send to the agent
        tools: Optional list of tools to provide to the agent

    Returns:
        Response from the agent
    """
    agent = get_agent(agents, agent_name)
    logger.info(f"Running agent: {agent_name}")

    if tools:
        return agent.run_with_tools(message, tools)
    else:
        return agent.run(message)


def run_coordinator(
    agents: Dict[str, adk.Agent], message: str, tools: Optional[List[adk.Tool]] = None
) -> Any:
    """Convenience function to run the coordinator agent.

    Args:
        agents: Dictionary of initialized agents
        message: Message to send to the coordinator
        tools: Optional list of tools to provide

    Returns:
        Response from the coordinator agent
    """
    return run_agent(agents, "coordinator", message, tools)
