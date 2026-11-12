import { AttributionDto, ResourceCapabilities } from "src/api/dto";
export enum ConnectionDetailsPageStateEnum {
  NEW = "NEW",
  LIST = "LIST",
  DETAIL = "DETAIL",
  EDIT = "EDIT",
}

export enum StatusType {
  Active = "ACTIVE",
  Inactive = "INACTIVE",
  Failed = "FAILED",
}

export type OfferedSourceTypeDto = {
  source_type: string;
  display_name: string;
};

export type ConnectionDto = {
  id: string;
  name: string;
  source_type: string;
  source_type_display_name: string;
  is_source_type_available: boolean;
  host: string;
  port: number | null;
  database: string;
  schema: string | null;
  username: string;
  description: string | null;
  status: StatusType;
  created_at: string;
  updated_at: string;
  capabilities: ResourceCapabilities;
  created_by: AttributionDto;
  modified_by: AttributionDto;
};
