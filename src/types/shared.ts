export type BootstrapStatus = {
  requiresSetup: boolean;
  clinic: ClinicSummary | null;
  license: LicenseStatus;
};

export type LicenseStatus = {
  status: "not_configured" | "trial_active" | "expired" | "licensed" | string;
  trialStartedAt?: string | null;
  trialEndsAt?: string | null;
  activatedAt?: string | null;
  daysRemaining: number;
  isTrialActive: boolean;
  isExpired: boolean;
  isLicensed: boolean;
  requiresActivation: boolean;
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
  license: LicenseStatus;
};

export type UserListItem = {
  id: string;
  fullName: string;
  username: string;
  roleName?: string | null;
};
