#!/usr/bin/env python
from constructs import Construct
from cdktf import App, TerraformStack, CloudBackend, NamedCloudWorkspace


class PatinaStack(TerraformStack):
    def __init__(self, scope: Construct, id: str):
        super().__init__(scope, id)

        # define resources here


app = App()
stack = PatinaStack(app, "patina-inf")
CloudBackend(stack,
  hostname='app.terraform.io',
  organization='Patina',
  workspaces=NamedCloudWorkspace('Patina')
)

app.synth()
