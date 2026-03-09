import { STORAGE_KEYS } from '@/lib/constants';
import { healthService } from './health.service';

/**
 * Authentication is handled via API key (secret_key) passed in Authorization header.
 * The backend expects: Authorization: Bearer <secret_key>
 *
 * There is no user/session management - just API key validation.
 */

export const authService = {
  /**
   * Set the API key for authentication
   */
  setApiKey: (apiKey: string) => {
    localStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, apiKey);
  },

  /**
   * Get the current API key
   */
  getApiKey: () => localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN),

  /**
   * Clear the API key (logout)
   */
  logout: () => {
    localStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
  },

  /**
   * Check if an API key is configured
   */
  isAuthenticated: () => !!localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN),

  /**
   * Validate the API key by making a test request to the health endpoint
   * Health endpoint doesn't require auth, but we can use it to verify server connectivity
   * Then try an authenticated endpoint to verify the key
   */
  validateApiKey: async (apiKey: string): Promise<boolean> => {
    // Store the current key to restore if validation fails
    const previousKey = localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN);

    try {
      // Temporarily set the key
      localStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, apiKey);

      // Try to access an authenticated endpoint
      const { instanceService } = await import('./instance.service');
      await instanceService.list();

      return true;
    } catch {
      // Restore previous key if validation failed
      if (previousKey) {
        localStorage.setItem(STORAGE_KEYS.AUTH_TOKEN, previousKey);
      } else {
        localStorage.removeItem(STORAGE_KEYS.AUTH_TOKEN);
      }
      return false;
    }
  },

  /**
   * Check server health (no auth required)
   */
  checkServerHealth: () => healthService.check(),
};
