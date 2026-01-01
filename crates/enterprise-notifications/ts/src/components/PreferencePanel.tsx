/**
 * PreferencePanel - User notification preferences management
 */

import React, { useState } from 'react';
import { NotificationPreference, NotificationChannelType, NotificationPriority } from '../types';

export interface PreferencePanelProps {
  preferences: NotificationPreference;
  onSave: (preferences: NotificationPreference) => void;
  className?: string;
}

export const PreferencePanel: React.FC<PreferencePanelProps> = ({
  preferences,
  onSave,
  className = '',
}) => {
  const [localPrefs, setLocalPrefs] = useState(preferences);

  const handleSave = () => {
    onSave(localPrefs);
  };

  return (
    <div className={`preference-panel ${className}`}>
      <h2>Notification Preferences</h2>

      {/* Global Settings */}
      <section className="pref-section">
        <h3>Global Settings</h3>
        <label>
          <input
            type="checkbox"
            checked={localPrefs.enabled}
            onChange={e =>
              setLocalPrefs({ ...localPrefs, enabled: e.target.checked })
            }
          />
          Enable notifications
        </label>
      </section>

      {/* Channel Preferences */}
      <section className="pref-section">
        <h3>Channels</h3>
        {localPrefs.channelPreferences.map((channelPref, index) => (
          <div key={channelPref.channel} className="channel-pref">
            <label>
              <input
                type="checkbox"
                checked={channelPref.enabled}
                onChange={e => {
                  const updated = [...localPrefs.channelPreferences];
                  updated[index] = { ...channelPref, enabled: e.target.checked };
                  setLocalPrefs({ ...localPrefs, channelPreferences: updated });
                }}
              />
              {channelPref.channel}
            </label>
          </div>
        ))}
      </section>

      {/* Quiet Hours */}
      <section className="pref-section">
        <h3>Quiet Hours</h3>
        <label>
          <input
            type="checkbox"
            checked={localPrefs.quietHours?.enabled ?? false}
            onChange={e => {
              setLocalPrefs({
                ...localPrefs,
                quietHours: {
                  ...localPrefs.quietHours!,
                  enabled: e.target.checked,
                  startTime: localPrefs.quietHours?.startTime ?? '22:00',
                  endTime: localPrefs.quietHours?.endTime ?? '08:00',
                },
              });
            }}
          />
          Enable quiet hours
        </label>

        {localPrefs.quietHours?.enabled && (
          <div className="quiet-hours-config">
            <label>
              Start:
              <input
                type="time"
                value={localPrefs.quietHours.startTime}
                onChange={e =>
                  setLocalPrefs({
                    ...localPrefs,
                    quietHours: { ...localPrefs.quietHours!, startTime: e.target.value },
                  })
                }
              />
            </label>
            <label>
              End:
              <input
                type="time"
                value={localPrefs.quietHours.endTime}
                onChange={e =>
                  setLocalPrefs({
                    ...localPrefs,
                    quietHours: { ...localPrefs.quietHours!, endTime: e.target.value },
                  })
                }
              />
            </label>
          </div>
        )}
      </section>

      {/* Digest Settings */}
      <section className="pref-section">
        <h3>Digest</h3>
        <label>
          <input
            type="checkbox"
            checked={localPrefs.digestEnabled}
            onChange={e =>
              setLocalPrefs({ ...localPrefs, digestEnabled: e.target.checked })
            }
          />
          Enable digest notifications
        </label>

        {localPrefs.digestEnabled && (
          <select
            value={localPrefs.digestFrequency}
            onChange={e =>
              setLocalPrefs({
                ...localPrefs,
                digestFrequency: e.target.value as 'hourly' | 'daily' | 'weekly',
              })
            }
          >
            <option value="hourly">Hourly</option>
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
          </select>
        )}
      </section>

      <button onClick={handleSave} className="save-btn">
        Save Preferences
      </button>
    </div>
  );
};

export default PreferencePanel;
