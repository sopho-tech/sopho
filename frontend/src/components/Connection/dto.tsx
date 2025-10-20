export const CONNECTOR_DISPLAY_TO_BACKEND_VALUE_MAP: Record<string, string> = {
  PostgreSQL: "POSTGRESQL",
  MySQL: "MYSQL",
  Supabase: "POSTGRESQL",
};

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

export type ConnectionDto = {
  id: string;
  name: string;
  source_type: string;
  host: string;
  port: number | null;
  database: string;
  schema: string | null;
  username: string;
  password: string;
  description: string | null;
  status: StatusType;
  created_at: string;
  updated_at: string;
};
