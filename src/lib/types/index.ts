export type Route = 'dashboard' | 'proxies' | 'settings';

export type ProxyProtocol = 'SOCKS5' | 'HTTP' | 'HTTPS';
export type ProxyStatus = 'active' | 'inactive' | 'error';
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

export interface Proxy {
  id: string;
  name: string;
  host: string;
  port: number;
  protocol: ProxyProtocol;
  country: string;
  countryCode: string;
  username?: string;
  password?: string;
  status: ProxyStatus;
  category?: string;
  lastUsedAt?: string;
}

export interface ConnectionState {
  status: ConnectionStatus;
  activeProxy: Proxy | null;
  currentIp: string | null;
  downloadSpeed: number;
  uploadSpeed: number;
}

export interface ConnectionSession {
  id: string;
  proxyId: string | null;
  connectedAt: string;
  disconnectedAt: string | null;
  ipAddress: string | null;
}
