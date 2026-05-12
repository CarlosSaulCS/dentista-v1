export type BootstrapStatus = {
  requiresSetup: boolean;
  clinic: ClinicSummary | null;
};

export type ClinicSummary = {
  id: string;
  name: string;
  subtitle: string;
  phone?: string | null;
  whatsapp?: string | null;
  email?: string | null;
  address?: string | null;
};

export type UserProfile = {
  id: string;
  clinicId: string;
  fullName: string;
  username: string;
  email?: string | null;
  roleName?: string | null;
  professionalLicense?: string | null;
  specialty?: string | null;
};

export type AuthSession = {
  sessionToken: string;
  expiresAt: string;
  user: UserProfile;
  permissions: string[];
};

export type UserListItem = {
  id: string;
  fullName: string;
  username: string;
  roleName?: string | null;
};
