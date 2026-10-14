import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "src/css/index.css";
import { Home } from "src/components/Home/Home";
import SignIn from "src/components/SignIn/SignIn";
import { Settings } from "src/components/Settings";
import { Connections } from "src/components/Connection/ConnectionsPage";
import { AIConfiguration } from "src/components/Settings/AIConfiguration";
import { Profile } from "src/components/Profile";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { TenantIndexRedirect, TenantRoute } from "src/components/TenantRoute";
import { Onboarding } from "src/components/Onboarding";
import { Members } from "src/components/Settings/Members";
import { createBrowserRouter, Navigate, RouterProvider } from "react-router";
import { ReactQueryClientProvider } from "./utils/react_query_provider";
import { APP_ROUTES } from "src/constants/app_routes";
import { Canvases } from "src/components/Canvases";
import { Canvas } from "src/components/Canvases/Canvas";
import { ConnectionNew } from "./components/Connection/ConnectionNew";
import { ConnectionEdit } from "./components/Connection/ConnectionEdit/ConnectionEdit";
import { ConversationalAnalytics } from "src/components/ConversationalAnalytics";
import { NewConversationPanel } from "./components/ConversationalAnalytics/NewConversationPanel";
import { ExistingConversationPanel } from "./components/ConversationalAnalytics/ExistingConversationPanel";
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
                path: "members",
                element: <Members />,
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
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ReactQueryClientProvider>
      <RouterProvider router={router} />
    </ReactQueryClientProvider>
  </StrictMode>,
);
