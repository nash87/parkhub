/**
 * ParkHub API Client
 */

const API_BASE = import.meta.env.VITE_API_URL || '';
const DEMO_MODE = !API_BASE || import.meta.env.VITE_DEMO === 'true';

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

class ApiClient {
  private token: string | null = null;

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('parkhub_token', token);
    } else {
      localStorage.removeItem('parkhub_token');
    }
  }

  getToken(): string | null {
    if (!this.token) {
      this.token = localStorage.getItem('parkhub_token');
    }
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...options.headers as Record<string, string>,
    };

    const token = this.getToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    try {
      const response = await fetch(`${API_BASE}${endpoint}`, {
        ...options,
        headers,
      });

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'NETWORK_ERROR',
          message: error instanceof Error ? error.message : 'Network error',
        },
      };
    }
  }

  // Auth
  async login(username: string, password: string) {
    if (DEMO_MODE) {
      return { success: true, data: { user: { id: 'demo-1', username, email: 'demo@parkhub.de', name: 'Max Mustermann', role: 'admin' as const, created_at: new Date().toISOString() }, tokens: { access_token: 'demo-token', refresh_token: 'demo-refresh', token_type: 'Bearer', expires_in: 86400 } } };
    }
    return this.request<{ user: User; tokens: AuthTokens }>('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
  }

  async register(data: RegisterData) {
    if (DEMO_MODE) {
      return { success: true, data: { user: { id: 'demo-' + Date.now(), username: data.username, email: data.email, name: data.name, role: 'user' as const, created_at: new Date().toISOString() }, tokens: { access_token: 'demo-token', refresh_token: 'demo-refresh', token_type: 'Bearer', expires_in: 86400 } } };
    }
    return this.request<{ user: User; tokens: AuthTokens }>('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async refreshToken(refreshToken: string) {
    return this.request<AuthTokens>('/api/v1/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  }

  // Users
  async getCurrentUser(): Promise<ApiResponse<User>> {
    if (DEMO_MODE && this.getToken()) {
      return { success: true, data: { id: 'demo-1', username: 'demo', email: 'demo@parkhub.de', name: 'Max Mustermann', role: 'admin', created_at: new Date().toISOString() } };
    }
    return this.request<User>('/api/v1/users/me');
  }

  // Lots
  async getLots(): Promise<ApiResponse<ParkingLot[]>> {
    if (DEMO_MODE) {
      return { success: true, data: [
        { id: 'lot-1', name: 'Firmenparkplatz', address: 'Hauptstraße 1', total_slots: 13, available_slots: 8 },
        { id: 'lot-2', name: 'Tiefgarage Nord', address: 'Nordring 5', total_slots: 24, available_slots: 15 },
      ]};
    }
    return this.request<ParkingLot[]>('/api/v1/lots');
  }

  async getLot(id: string) {
    return this.request<ParkingLot>(`/api/v1/lots/${id}`);
  }

  async getLotSlots(lotId: string): Promise<ApiResponse<ParkingSlot[]>> {
    if (DEMO_MODE) {
      return { success: true, data: Array.from({ length: 13 }, (_, i) => ({
        id: `slot-${lotId}-${i}`, lot_id: lotId, number: String(45 + i),
        status: (i === 2 || i === 7) ? 'occupied' as const : i === 4 ? 'reserved' as const : 'available' as const,
        floor: 0, section: i < 6 ? 'A' : 'B',
      }))};
    }
    return this.request<ParkingSlot[]>(`/api/v1/lots/${lotId}/slots`);
  }

  // Bookings
  async getBookings(): Promise<ApiResponse<Booking[]>> {
    if (DEMO_MODE) {
      const now = Date.now();
      return { success: true, data: [
        { id: 'b1', user_id: 'demo-1', slot_id: 'slot-a-2', lot_id: 'lot-1', slot_number: '47', lot_name: 'Firmenparkplatz', vehicle_plate: 'M-AB 1234', start_time: new Date(now - 1800000).toISOString(), end_time: new Date(now + 3600000).toISOString(), status: 'active', booking_type: 'einmalig', created_at: new Date().toISOString() },
        { id: 'b2', user_id: 'demo-1', slot_id: 'slot-tg-a-1', lot_id: 'lot-2', slot_number: '2', lot_name: 'Tiefgarage Nord', vehicle_plate: 'M-AB 1234', start_time: new Date(now - 86400000).toISOString(), end_time: new Date(now - 82800000).toISOString(), status: 'completed', booking_type: 'einmalig', created_at: new Date(now - 86400000).toISOString() },
        { id: 'b3', user_id: 'demo-1', slot_id: 'slot-a-0', lot_id: 'lot-1', slot_number: '45', lot_name: 'Firmenparkplatz', vehicle_plate: 'M-CD 5678', start_time: new Date(now + 86400000).toISOString(), end_time: new Date(now + 86400000 * 4).toISOString(), status: 'active', booking_type: 'mehrtaegig', created_at: new Date().toISOString() },
        { id: 'b4', user_id: 'demo-1', slot_id: 'slot-b-0', lot_id: 'lot-1', slot_number: '51', lot_name: 'Firmenparkplatz', vehicle_plate: 'M-AB 1234', start_time: new Date(now - 604800000).toISOString(), end_time: new Date(now + 604800000 * 3).toISOString(), status: 'active', booking_type: 'dauer', dauer_interval: 'monthly', created_at: new Date(now - 604800000).toISOString() },
        { id: 'b5', user_id: 'demo-1', slot_id: 'slot-tg-b-2', lot_id: 'lot-2', slot_number: '11', lot_name: 'Tiefgarage Nord', vehicle_plate: 'M-CD 5678', start_time: new Date(now - 172800000).toISOString(), end_time: new Date(now - 169200000).toISOString(), status: 'completed', booking_type: 'einmalig', created_at: new Date(now - 172800000).toISOString() },
      ]};
    }
    return this.request<Booking[]>('/api/v1/bookings');
  }

  async createBooking(data: CreateBookingData) {
    if (DEMO_MODE) {
      return {
        success: true,
        data: {
          id: 'booking-' + Date.now(),
          user_id: 'demo-1',
          slot_id: data.slot_id,
          lot_id: 'lot-1',
          slot_number: '47',
          lot_name: 'Firmenparkplatz',
          vehicle_plate: data.license_plate || 'M-AB 1234',
          start_time: data.start_time,
          end_time: new Date(new Date(data.start_time).getTime() + (data.duration_minutes || 60) * 60000).toISOString(),
          status: 'active' as const,
          created_at: new Date().toISOString(),
        },
      };
    }
    return this.request<Booking>('/api/v1/bookings', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async cancelBooking(id: string) {
    return this.request<void>(`/api/v1/bookings/${id}`, {
      method: 'DELETE',
    });
  }

  // Vehicles
  async getVehicles(): Promise<ApiResponse<Vehicle[]>> {
    if (DEMO_MODE) {
      return { success: true, data: [
        { id: 'v1', user_id: 'demo-1', license_plate: 'M-AB 1234', make: 'BMW', model: '320i', color: 'Schwarz', photoUrl: generateCarPhotoSvg('Schwarz'), is_default: true },
        { id: 'v2', user_id: 'demo-1', license_plate: 'M-CD 5678', make: 'VW', model: 'Golf', color: 'Weiß', photoUrl: generateCarPhotoSvg('Weiß'), is_default: false },
      ]};
    }
    return this.request<Vehicle[]>('/api/v1/vehicles');
  }

  async createVehicle(data: CreateVehicleData) {
    return this.request<Vehicle>('/api/v1/vehicles', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async deleteVehicle(id: string) {
    return this.request<void>(`/api/v1/vehicles/${id}`, {
      method: 'DELETE',
    });
  }

  async uploadVehiclePhoto(vehicleId: string, file: File): Promise<ApiResponse<{ url: string }>> {
    if (DEMO_MODE) {
      const url = URL.createObjectURL(file);
      return { success: true, data: { url } };
    }
    const formData = new FormData();
    formData.append('photo', file);
    return this.request(`/api/v1/vehicles/${vehicleId}/photo`, { method: 'POST', body: formData });
  }

  // Lot detailed (mock)
  async getLotDetailed(id: string): Promise<ApiResponse<ParkingLotDetailed>> {
    if (id === 'lot-2') {
      return {
        success: true,
        data: {
          id: 'lot-2',
          name: 'Tiefgarage Nord',
          address: 'Nordring 5',
          total_slots: 22,
          available_slots: 15,
          layout: {
            roadLabel: 'Fahrweg',
            rows: [
              {
                id: 'row-tg-a',
                side: 'top',
                label: 'Reihe A',
                slots: Array.from({ length: 8 }, (_, i) => ({
                  id: `slot-tg-a-${i}`,
                  number: String(1 + i),
                  status: (i === 3 ? 'occupied' : i === 6 ? 'reserved' : 'available') as SlotConfig['status'],
                  vehiclePlate: i === 3 ? 'B-KL 4455' : undefined,
                })),
              },
              {
                id: 'row-tg-b',
                side: 'bottom',
                label: 'Reihe B',
                slots: Array.from({ length: 8 }, (_, i) => ({
                  id: `slot-tg-b-${i}`,
                  number: String(9 + i),
                  status: (i === 0 ? 'occupied' : i === 4 ? 'occupied' : i === 7 ? 'disabled' : 'available') as SlotConfig['status'],
                  vehiclePlate: i === 0 ? 'F-GH 7890' : i === 4 ? 'K-MN 3456' : undefined,
                })),
              },
              {
                id: 'row-tg-c',
                side: 'bottom',
                label: 'Reihe C',
                slots: Array.from({ length: 6 }, (_, i) => ({
                  id: `slot-tg-c-${i}`,
                  number: String(17 + i),
                  status: (i === 2 ? 'occupied' : i === 5 ? 'reserved' : 'available') as SlotConfig['status'],
                  vehiclePlate: i === 2 ? 'D-QR 1122' : undefined,
                })),
              },
            ],
          },
        },
      };
    }
    return {
      success: true,
      data: {
        id: 'lot-1',
        name: 'Firmenparkplatz',
        address: 'Hauptstraße 1',
        total_slots: 13,
        available_slots: 8,
        layout: {
          roadLabel: 'Fahrweg',
          rows: [
            {
              id: 'row-a',
              side: 'top',
              label: 'Reihe A',
              slots: Array.from({ length: 6 }, (_, i) => ({
                id: `slot-a-${i}`,
                number: String(45 + i),
                status: (i === 2 ? 'occupied' : i === 4 ? 'reserved' : i === 5 ? 'homeoffice' : 'available') as SlotConfig['status'],
                vehiclePlate: i === 2 ? 'M-AB 1234' : undefined,
                homeofficeUser: i === 5 ? 'Lisa S.' : undefined,
              })),
            },
            {
              id: 'row-b',
              side: 'bottom',
              label: 'Reihe B',
              slots: Array.from({ length: 7 }, (_, i) => ({
                id: `slot-b-${i}`,
                number: String(51 + i),
                status: (i === 1 ? 'occupied' : i === 5 ? 'occupied' : i === 3 ? 'disabled' : i === 6 ? 'homeoffice' : 'available') as SlotConfig['status'],
                vehiclePlate: i === 1 ? 'S-XY 5678' : i === 5 ? 'HH-CD 9012' : undefined,
                homeofficeUser: i === 6 ? 'Max M.' : undefined,
              })),
            },
          ],
        },
      },
    };
  }

  // Homeoffice
  async getHomeofficeSettings(): Promise<ApiResponse<HomeofficeSettings>> {
    if (DEMO_MODE) {
      return { success: true, data: {
        pattern: { weekdays: [0, 2] },
        singleDays: [
          { id: 'ho-1', date: '2026-02-13' },
          { id: 'ho-2', date: '2026-02-20' },
        ],
        parkingSlot: { number: '47', lotName: 'Firmenparkplatz' },
      }};
    }
    return this.request('/api/v1/homeoffice');
  }

  async updateHomeofficePattern(weekdays: number[]): Promise<ApiResponse<void>> {
    if (DEMO_MODE) return { success: true };
    return this.request('/api/v1/homeoffice/pattern', { method: 'PUT', body: JSON.stringify({ weekdays }) });
  }

  async addHomeofficeDay(date: string, reason?: string): Promise<ApiResponse<HomeofficeDay>> {
    if (DEMO_MODE) return { success: true, data: { id: `ho-${Date.now()}`, date, reason } };
    return this.request('/api/v1/homeoffice/days', { method: 'POST', body: JSON.stringify({ date, reason }) });
  }

  async removeHomeofficeDay(id: string): Promise<ApiResponse<void>> {
    if (DEMO_MODE) return { success: true };
    return this.request(`/api/v1/homeoffice/days/${id}`, { method: 'DELETE' });
  }

  // Health
  async health() {
    return this.request<{ status: string }>('/health');
  }
}

export const api = new ApiClient();

// Types
export interface User {
  id: string;
  username: string;
  email: string;
  name: string;
  role: 'user' | 'admin' | 'superadmin';
  created_at: string;
}

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface RegisterData {
  username: string;
  email: string;
  password: string;
  name: string;
}

export interface ParkingLot {
  id: string;
  name: string;
  address: string;
  total_slots: number;
  available_slots: number;
}

export interface ParkingSlot {
  id: string;
  lot_id: string;
  number: string;
  status: 'available' | 'occupied' | 'reserved' | 'disabled';
  floor?: number;
  section?: string;
}

export interface Booking {
  id: string;
  user_id: string;
  slot_id: string;
  lot_id: string;
  slot_number: string;
  lot_name: string;
  vehicle_plate?: string;
  start_time: string;
  end_time: string;
  status: 'active' | 'completed' | 'cancelled';
  booking_type?: 'einmalig' | 'mehrtaegig' | 'dauer';
  dauer_interval?: 'weekly' | 'monthly';
  created_at: string;
}

export interface CreateBookingData {
  slot_id: string;
  start_time: string;
  duration_minutes: number;
  vehicle_id?: string;
  license_plate?: string;
}

export interface Vehicle {
  id: string;
  user_id: string;
  license_plate: string;
  make?: string;
  model?: string;
  color?: string;
  photoUrl?: string;
  is_default: boolean;
}

// Generate SVG car placeholder with given color
export function generateCarPhotoSvg(color: string): string {
  const colorHex: Record<string, string> = {
    'Schwarz': '#1f2937', 'Weiß': '#e5e7eb', 'Silber': '#9ca3af', 'Grau': '#6b7280',
    'Blau': '#2563eb', 'Rot': '#dc2626', 'Grün': '#16a34a', 'Gelb': '#eab308',
  };
  const bg = colorHex[color] || '#6b7280';
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="0 0 200 200">
    <rect width="200" height="200" rx="20" fill="${bg}"/>
    <g transform="translate(40,50)" fill="rgba(255,255,255,0.3)">
      <path d="M10,70 L20,35 Q25,20 40,15 L80,10 Q95,8 105,15 L115,25 Q120,30 120,40 L120,70 Q120,80 110,80 L20,80 Q10,80 10,70 Z"/>
      <rect x="20" y="75" width="25" height="12" rx="6" fill="rgba(255,255,255,0.25)"/>
      <rect x="85" y="75" width="25" height="12" rx="6" fill="rgba(255,255,255,0.25)"/>
      <rect x="35" y="30" width="22" height="18" rx="4" fill="rgba(255,255,255,0.15)"/>
      <rect x="65" y="28" width="22" height="18" rx="4" fill="rgba(255,255,255,0.15)"/>
    </g>
  </svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

export interface CreateVehicleData {
  license_plate: string;
  make?: string;
  model?: string;
  color?: string;
}

// Parking lot layout configuration
export interface LotLayout {
  rows: LotRow[];
  roadLabel?: string;
}

export interface LotRow {
  id: string;
  side: 'top' | 'bottom';
  slots: SlotConfig[];
  label?: string;
}

export interface SlotConfig {
  id: string;
  number: string;
  status: 'available' | 'occupied' | 'reserved' | 'disabled' | 'blocked' | 'homeoffice';
  vehiclePlate?: string;
  bookedBy?: string;
  homeofficeUser?: string;
}

export interface ParkingLotDetailed extends ParkingLot {
  layout?: LotLayout;
}

// Homeoffice types
export interface HomeofficePattern {
  weekdays: number[]; // 0=Mon, 1=Tue, ... 4=Fri
}

export interface HomeofficeDay {
  id: string;
  date: string; // ISO date
  reason?: string;
}

export interface HomeofficeSettings {
  pattern: HomeofficePattern;
  singleDays: HomeofficeDay[];
  parkingSlot?: { number: string; lotName: string };
}
