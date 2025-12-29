/**
 * Provider Setup Component
 * Step-by-step guide for SSO provider setup
 */

import React, { useState } from 'react';
import { SSOProvider } from '../../types';

interface ProviderSetupProps {
  provider: SSOProvider;
  organizationDomain: string;
  className?: string;
}

export const ProviderSetup: React.FC<ProviderSetupProps> = ({
  provider,
  organizationDomain,
  className,
}) => {
  const [activeStep, setActiveStep] = useState(0);

  const getSetupSteps = () => {
    switch (provider) {
      case SSOProvider.SAML:
        return [
          {
            title: 'Configure Identity Provider',
            content: (
              <div>
                <p>Add this application to your SAML identity provider with the following settings:</p>
                <ul>
                  <li><strong>ACS URL:</strong> <code>https://{organizationDomain}/auth/saml/callback</code></li>
                  <li><strong>Entity ID:</strong> <code>https://{organizationDomain}</code></li>
                  <li><strong>Name ID Format:</strong> Email Address</li>
                </ul>
              </div>
            ),
          },
          {
            title: 'Download Metadata',
            content: (
              <div>
                <p>Download the SAML metadata XML from your identity provider or copy:</p>
                <ul>
                  <li>Entity ID</li>
                  <li>SSO URL</li>
                  <li>X.509 Certificate</li>
                </ul>
              </div>
            ),
          },
          {
            title: 'Configure Attributes',
            content: (
              <div>
                <p>Ensure your identity provider sends these attributes:</p>
                <ul>
                  <li><strong>email</strong> (required)</li>
                  <li><strong>firstName</strong> (optional)</li>
                  <li><strong>lastName</strong> (optional)</li>
                  <li><strong>displayName</strong> (optional)</li>
                </ul>
              </div>
            ),
          },
          {
            title: 'Test Connection',
            content: (
              <div>
                <p>After configuring, use the "Test Configuration" button to verify the setup.</p>
              </div>
            ),
          },
        ];

      case SSOProvider.OIDC:
        return [
          {
            title: 'Register Application',
            content: (
              <div>
                <p>Register your application with your OIDC provider using:</p>
                <ul>
                  <li><strong>Redirect URI:</strong> <code>https://{organizationDomain}/auth/oidc/callback</code></li>
                  <li><strong>Scopes:</strong> openid, profile, email</li>
                </ul>
              </div>
            ),
          },
          {
            title: 'Get Credentials',
            content: (
              <div>
                <p>Copy from your OIDC provider:</p>
                <ul>
                  <li>Client ID</li>
                  <li>Client Secret</li>
                  <li>Issuer URL</li>
                  <li>Authorization URL</li>
                  <li>Token URL</li>
                  <li>User Info URL</li>
                </ul>
              </div>
            ),
          },
          {
            title: 'Test Connection',
            content: (
              <div>
                <p>Use the "Test Configuration" button to verify the OIDC setup.</p>
              </div>
            ),
          },
        ];

      default:
        return [
          {
            title: 'Setup Guide',
            content: <div><p>Please refer to your provider's documentation for setup instructions.</p></div>,
          },
        ];
    }
  };

  const steps = getSetupSteps();

  return (
    <div className={className}>
      <header>
        <h2>{provider} Setup Guide</h2>
        <p className="subtitle">Follow these steps to configure SSO</p>
      </header>

      <div className="setup-wizard">
        <div className="steps-indicator">
          {steps.map((step, index) => (
            <button
              key={index}
              type="button"
              onClick={() => setActiveStep(index)}
              className={`step-button ${index === activeStep ? 'active' : ''} ${
                index < activeStep ? 'completed' : ''
              }`}
              aria-current={index === activeStep ? 'step' : undefined}
            >
              <span className="step-number">{index + 1}</span>
              <span className="step-title">{step.title}</span>
            </button>
          ))}
        </div>

        <div className="step-content">
          <h3>{steps[activeStep].title}</h3>
          {steps[activeStep].content}
        </div>

        <div className="step-navigation">
          {activeStep > 0 && (
            <button
              type="button"
              onClick={() => setActiveStep(activeStep - 1)}
              className="btn btn-secondary"
            >
              Previous
            </button>
          )}
          {activeStep < steps.length - 1 && (
            <button
              type="button"
              onClick={() => setActiveStep(activeStep + 1)}
              className="btn btn-primary"
            >
              Next
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProviderSetup;
