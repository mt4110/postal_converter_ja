terraform {
  required_version = ">= 1.6.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.0"
    }
  }
}

provider "azurerm" {
  features {}
}

locals {
  platform = "azure"
}

#Node: module "network" {
#Node:   source      = "../../modules/azure/network"
#Node:   location    = var.location
#Node:   environment = var.environment
#Node: }
#
#Node: module "container_runtime" {
#Node:   source       = "../../modules/azure/containerapps"
#Node:   service_name = var.service_name
#Node: }

output "skeleton_summary" {
  value = "terraform skeleton ready: platform=${local.platform}, env=${var.environment}, location=${var.location}"
}
