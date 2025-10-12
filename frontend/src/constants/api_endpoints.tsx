export const API_ENDPOINTS = {
  AUTH: {
    CREATE_SESSION: "/api/v1/auth/session",
    GET_SESSION: "/api/v1/auth/session",
  },
  CONNECTION: {
    CREATE: "/api/v1/connection",
    GET_ALL: "/api/v1/connection",
    GET_BY_ID: "/api/v1/connection/:id",
    UPDATE: "/api/v1/connection/:id",
    DELETE: "/api/v1/connection/:id",
  },
  NOTEBOOK: {
    CREATE: "/api/v1/notebook",
    GET_ALL: "/api/v1/notebook",
    GET_BY_ID: "/api/v1/notebook/:id",
    UPDATE: "/api/v1/notebook/:id",
    DELETE: "/api/v1/notebook/:id",
  },
  CELL: {
    CREATE: "/api/v1/cell",
    GET_ALL: "/api/v1/cell",
    GET_BY_ID: "/api/v1/cell/:id",
    UPDATE: "/api/v1/cell/:id",
    DELETE: "/api/v1/cell/:id",
    EXECUTE: "/api/v1/cell/:id/execute",
  },
};
