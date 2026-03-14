import { NavLink } from "react-router-dom";
import {
  Calendar,
  FolderOpen,
  LayoutDashboard,
  Settings,
  Video,
} from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";

const navItems = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/recordings", label: "Recordings", icon: Video },
  { to: "/schedules", label: "Schedules", icon: Calendar },
  { to: "/categories", label: "Categories", icon: FolderOpen },
  { to: "/settings", label: "Settings", icon: Settings },
];

export default function Sidebar() {
  return (
    <aside className="flex h-screen w-64 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div className="flex h-14 items-center px-4 font-semibold tracking-tight">
        BBB Library
      </div>
      <Separator />
      <ScrollArea className="flex-1 px-2 py-2">
        <nav className="flex flex-col gap-1">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                `flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
                }`
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
      </ScrollArea>
    </aside>
  );
}
