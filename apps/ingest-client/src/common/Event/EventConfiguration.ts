export interface EventConfiguration {
  api_key: string;
  host: string;
}

export enum EventConfigurationState {
  Invalid,
  InvalidApiKey,
  InvalidHost,
  Success,
}

export function validateEventConfiguration(
  config: EventConfiguration,
): EventConfigurationState {
  if (config.api_key.trim().length < 1) {
    return EventConfigurationState.InvalidApiKey;
  }
  const host = URL.parse(config.host);
  if (host == null) {
    return EventConfigurationState.InvalidHost;
  }
  if (
    host.pathname.length > 1 ||
    host.search.length > 0 ||
    host.hash.length > 0
  ) {
    return EventConfigurationState.InvalidHost;
  }
  return EventConfigurationState.Success;
}
