import { useState } from "react";
import { Outlet, Link, useLocation } from "react-router-dom";

/* Example icons – replace with Lucide components */
import { Home, User, Leaf } from "lucide-react";

const navigation: {
  name: string;
  href: string;
  icon?: React.ComponentType<{ className?: string }>;
}[] = [
  { name: "Plants", href: "/", icon: Leaf },
  { name: "Profile", href: "#", icon: User },
];

function classNames(...classes: (string | false | undefined)[]) {
  return classes.filter(Boolean).join(" ");
}

export default function Shell() {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const location = useLocation();

  return (
    <div className="min-h-screen bg-green-50">
      {/* Mobile overlay */}
      {sidebarOpen && (
        <div className="fixed inset-0 z-40 flex lg:hidden">
          <div
            className="fixed inset-0 bg-black/40"
            onClick={() => setSidebarOpen(false)}
          />
          <aside className="relative z-50 w-64 bg-white text-green-900 flex flex-col shadow-lg">
            <div className="flex h-16 items-center justify-between px-4 border-b border-green-100">
              <img src="/logo.jpeg" alt="Mycelium" className="h-10 w-auto" />
              <button onClick={() => setSidebarOpen(false)}>
                <svg
                  viewBox="0 0 24 24"
                  className="h-6 w-6"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>

            <nav className="flex-1 px-3 py-4 space-y-1">
              {navigation.map((item) => {
                const isActive = location.pathname === item.href;
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    onClick={() => setSidebarOpen(false)}
                    className={classNames(
                      isActive
                        ? "bg-green-100 text-green-900 font-semibold"
                        : "text-green-900 hover:bg-green-100",
                      "flex items-center gap-2 rounded-md px-3 py-2 text-sm"
                    )}
                  >
                    {item.icon && <item.icon className="h-5 w-5" />}
                    {item.name}
                  </Link>
                );
              })}
            </nav>
          </aside>
        </div>
      )}

      {/* Desktop sidebar */}
      <aside className="hidden lg:fixed lg:inset-y-0 lg:z-30 lg:flex lg:w-72 lg:flex-col bg-white text-green-900 shadow">
        <div className="flex items-center border-b border-green-100">
          <img src="/logo.jpeg" alt="Mycelium" className="h-32 p-2 w-auto mx-auto" />
        </div>

        <nav className="flex-1 px-4 py-6 space-y-1">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href;
            return (
              <Link
                key={item.name}
                to={item.href}
                className={classNames(
                  isActive
                    ? "bg-green-100 text-green-900 font-semibold"
                    : "text-green-900 hover:bg-green-100",
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm"
                )}
              >
                {item.icon && <item.icon className="h-5 w-5" />}
                {item.name}
              </Link>
            );
          })}
        </nav>
      </aside>

      {/* Top bar (mobile) */}
      <header className="sticky top-0 z-20 flex h-16 items-center gap-x-4 bg-white px-4 shadow lg:hidden">
        <button
          onClick={() => setSidebarOpen(true)}
          className="text-green-900"
        >
          <svg
            viewBox="0 0 24 24"
            className="h-6 w-6"
            fill="none"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M4 6h16M4 12h16M4 18h16"
            />
          </svg>
        </button>
        <span className="text-sm font-semibold text-green-900">
          Your plants
        </span>
      </header>

      {/* Main content */}
      <main className="lg:pl-72 bg-green-50 min-h-screen">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
