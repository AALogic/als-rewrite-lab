export const WETRANSFER_PROVIDER_ID = "wetransfer_web" as const;

export type ExternalProviderOpenRequest = {
  request_id: string;
  provider_id: typeof WETRANSFER_PROVIDER_ID;
};

export type ExternalProviderOpenResult = {
  schema_version: "0.1";
  request_id: string;
  provider_id: typeof WETRANSFER_PROVIDER_ID;
  state: "opened";
};

export type ExternalProviderOpenError = {
  error_code: string;
  stage: string;
  message: string;
};
