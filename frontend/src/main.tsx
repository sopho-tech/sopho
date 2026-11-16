import {
  PlatformAIUsage,
  PlatformUsageByConversation,
  PlatformUsageByTenant,
  PlatformUsageByUser,
  PlatformUsageByWork,
  PlatformUsageConversationQuestions,
  PlatformUsageSummary,
} from "src/components/Platform/AIUsage";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "src/css/index.css";
import { Home } from "src/components/Home/Home";
import SignIn from "src/components/SignIn/SignIn";
import { InvitationLink } from "src/components/InvitationLink";
import {
  ForgotPassword,
  ResetPassword,
  requirePasswordSignIn,
} from "src/components/PasswordReset";
import { Settings } from "src/components/Settings";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { AIConfiguration } from "src/components/Settings/AIConfiguration";
import {
  AIUsage,
  UsageByConversation,
  UsageByUser,
  UsageByWork,
  UsageConversationQuestions,
  UsageSummary,
} from "src/components/Settings/AIUsage";
import { Profile, ProfileRedirect } from "src/components/Profile";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { TenantIndexRedirect, TenantRoute } from "src/components/TenantRoute";
import { Onboarding } from "src/components/Onboarding";
import { Invitations, Members } from "src/components/Settings/Members";
import {
  EmailTemplates,
  PlatformIndexRedirect,
  PlatformRoute,
  SentEmails,
  Tenants,
  Users,
} from "src/components/Platform";
import { createBrowserRouter, Navigate, RouterProvider } from "react-router";
import { ReactQueryClientProvider } from "src/utils/react_query_provider";
import { APP_ROUTES } from "src/constants/app_routes";
import { Canvases } from "src/components/Canvases";
import { Canvas } from "src/components/Canvases/Canvas";
import { ConnectionNew } from "src/components/Connection/ConnectionNew";
import { ConnectionEdit } from "src/components/Connection/ConnectionEdit/ConnectionEdit";
import { ConversationalAnalytics } from "src/components/ConversationalAnalytics";
import { NewConversationPanel } from "src/components/ConversationalAnalytics/NewConversationPanel";
import { ExistingConversationPanel } from "src/components/ConversationalAnalytics/ExistingConversationPanel";
import { ConversationList } from "src/components/ConversationalAnalytics/ConversationList";
import { Notebook } from "src/components/Notebook";
import { Dashboard } from "src/components/Dashboard";
import { CanvasIndexRedirect } from "src/components/Canvases/Canvas";

const router = createBrowserRouter([
  {
    element: <ProtectedRoute />,
    children: [
      {
        path: APP_ROUTES.INDEX,
        element: <TenantIndexRedirect />,
      },
      {
        path: APP_ROUTES.ONBOARDING,
        element: <Onboarding />,
      },
      {
        path: APP_ROUTES.PROFILE,
        element: <ProfileRedirect />,
      },
      {
        path: APP_ROUTES.PLATFORM,
        element: <PlatformRoute />,
        children: [
          {
            index: true,
            element: <PlatformIndexRedirect />,
          },
          {
            path: "users",
            element: <Users />,
          },
          {
            path: "tenants",
            element: <Tenants />,
          },
          {
            path: "ai-usage",
            element: <PlatformAIUsage />,
            children: [
              {
                index: true,
                element: <Navigate to="summary" replace />,
              },
              {
                path: "summary",
                element: <PlatformUsageSummary />,
              },
              {
                path: "by-tenant",
                element: <PlatformUsageByTenant />,
              },
              {
                path: "by-user",
                element: <PlatformUsageByUser />,
              },
              {
                path: "by-work",
                element: <PlatformUsageByWork />,
              },
              {
                path: "by-conversation",
                element: <PlatformUsageByConversation />,
              },
              {
                path: "by-conversation/:usageTenantId/:conversationId",
                element: <PlatformUsageConversationQuestions />,
              },
            ],
          },
          {
            path: "emails",
            children: [
              {
                index: true,
                element: <Navigate to="sent" replace />,
              },
              {
                path: "sent",
                element: <SentEmails />,
              },
              {
                path: "templates",
                element: <EmailTemplates />,
              },
            ],
          },
        ],
      },
      {
        path: APP_ROUTES.TENANT,
        element: <TenantRoute />,
        children: [
          {
            index: true,
            element: <Home />,
          },
          {
            path: "conversational_analytics",
            element: <ConversationalAnalytics />,
            children: [
              {
                index: true,
                element: <NewConversationPanel />,
              },
              {
                path: "conversation/:id",
                element: <ExistingConversationPanel />,
              },
              {
                path: "conversations",
                element: <ConversationList />,
              },
            ],
          },
          {
            path: "settings",
            element: <Settings />,
            children: [
              {
                index: true,
                element: <Navigate to="connections" replace />,
              },
              {
                path: "connections",
                element: <Connections />,
              },
              {
                path: "ai-configurations",
                element: <AIConfiguration />,
              },
              {
                path: "ai-usage",
                element: <AIUsage />,
                children: [
                  {
                    index: true,
                    element: <Navigate to="summary" replace />,
                  },
                  {
                    path: "summary",
                    element: <UsageSummary />,
                  },
                  {
                    path: "by-user",
                    element: <UsageByUser />,
                  },
                  {
                    path: "by-work",
                    element: <UsageByWork />,
                  },
                  {
                    path: "by-conversation",
                    element: <UsageByConversation />,
                  },
                  {
                    path: "by-conversation/:conversationId",
                    element: <UsageConversationQuestions />,
                  },
                ],
              },
              {
                path: "members",
                element: <Members />,
              },
              {
                path: "invitations",
                element: <Invitations />,
              },
              {
                path: "connection/new",
                element: <ConnectionNew />,
              },
              {
                path: "connection/:id/edit",
                element: <ConnectionEdit />,
              },
            ],
          },
          {
            path: "profile",
            element: <Profile />,
          },
          {
            path: "canvas",
            element: <Canvases />,
          },
          {
            path: "canvas/:id",
            element: <Canvas />,
            children: [
              {
                index: true,
                element: <CanvasIndexRedirect />,
              },
              {
                path: APP_ROUTES.CANVAS_ROUTES.NOTEBOOK,
                element: <Notebook />,
              },
              {
                path: APP_ROUTES.CANVAS_ROUTES.DASHBOARD,
                element: <Dashboard />,
              },
            ],
          },
        ],
      },
    ],
  },
  {
    path: APP_ROUTES.SIGN_IN,
    element: <SignIn />,
  },
  {
    path: APP_ROUTES.INVITATION,
    element: <InvitationLink />,
  },
  {
    path: APP_ROUTES.FORGOT_PASSWORD,
    loader: requirePasswordSignIn,
    element: <ForgotPassword />,
  },
  {
    path: APP_ROUTES.RESET_PASSWORD,
    loader: requirePasswordSignIn,
    element: <ResetPassword />,
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ReactQueryClientProvider>
      <RouterProvider router={router} />
    </ReactQueryClientProvider>
  </StrictMode>,
);
