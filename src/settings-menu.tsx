import React, { useState} from 'react';
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import gear from '@/assets/gear-white.svg';
import { Label } from '@radix-ui/react-dropdown-menu';

import { DispenseType, IchibuState, User } from '@/types';
import { invoke } from '@tauri-apps/api/core';


interface SettingsMenuProps {
  currentUser: User;
  currentDispenseType: DispenseType;
  setDispenseType: (type: DispenseType) => void;
}

const SettingsMenu: React.FC<SettingsMenuProps> = ({ currentUser, currentDispenseType, setDispenseType }) => {
  const superVisibility = currentUser === User.Admin || currentUser === User.Manager;
  
  // State to track whether the dropdown is open or closed
  const [open, setOpen] = useState(false);
  
  const handleToggle = (checked: Boolean) => {
    setDispenseType(checked ? DispenseType.LargeSmall : DispenseType.Classic);
  };

  const handleTimeoutReset = async () => {
    try {
      await invoke("clear_dispenser_time_out");
    } catch (error) {
      console.error("failed to clear dispenser timeout: ", error);
    }
  }
  const handleButton = async (state: IchibuState) => {
    try {
      await invoke("update_run_state", { newState: state });
    } catch (error) {
      console.error("Failed to set state:", error);
    }
  };

  // Prevent dropdown from closing when a menu item is clicked
  const handleItemClick = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  return (
    <div>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger asChild>
          <Button
            className="bg-slate-950 active:bg-slate-950 hover:bg-slate-950 focus:outline-none focus:ring-0 border-0 h-30"
          >
            <img src={gear} alt="settings" className="h-20" />
          </Button>
        </DropdownMenuTrigger>

        <DropdownMenuContent 
          className="bg-gray-600 h-full"
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            if (open) setOpen(true);
          }}
        >
          {superVisibility && (
            <div onClick={handleItemClick} className="px-2 py-1.5">
              <div className="flex items-center space-x-2 h-32">
                <Switch
                  checked={currentDispenseType === DispenseType.LargeSmall}
                  onCheckedChange={handleToggle}
                  id="dispense-mode"
                  className='h-8 w-16 cursor-none data-[state=checked]:bg-blue-300'
                />
                <Label className="text-4xl font-bold text-white">Sized Dispense</Label>
              </div>
            </div>
          )}
          <div onClick={handleItemClick} className="px-2 py-1.5">
            <Button 
              className="w-full text-4xl h-32 bg-blue-500"
              onClick={() => handleTimeoutReset()}
            >
              Refill Hopper
            </Button>
          </div>
          <div onClick={handleItemClick} className="px-2 py-1.5">
            <Button 
              className="w-full text-4xl h-32 bg-blue-500"
              onClick={() => handleButton(IchibuState.Cleaning)}
            >
              Clean Mode
            </Button>
          </div>
          <div onClick={handleItemClick} className="px-2 py-1.5">
            <Button 
              className="w-full h-32 text-4xl bg-green-600"
              onClick={() => handleButton(IchibuState.Emptying)}
            >
              Empty Hopper Start
            </Button>
          </div>
          <div onClick={handleItemClick} className="px-2 py-1.5">
            <Button 
              className="w-full h-32 text-4xl break-words bg-destructive"
              onClick={() => handleButton(IchibuState.Cleaning)}
            >
              Empty Hopper Stop
            </Button>
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

export default SettingsMenu;

// import React, { useEffect, useRef, useState } from 'react';
// import{ Button } from "@/components/ui/button";
// import { Switch } from "@/components/ui/switch"
// import {
//     DropdownMenu,
//     DropdownMenuContent,
//     DropdownMenuItem,
//     DropdownMenuTrigger,
//   } from "@/components/ui/dropdown-menu";

// import gear from '@/assets/gear-white.svg';
// import { Label } from '@radix-ui/react-dropdown-menu';


// import { DispenseType, IchibuState, User } from '@/types';
// import { invoke } from '@tauri-apps/api/core';

// interface SettingsMenuProps{
//     currentUser: User,
//     currentDispenseType: DispenseType
//     setDispenseType: (type: DispenseType) => void;
// }

// const SettingsMenu: React.FC<SettingsMenuProps> = ({currentUser, currentDispenseType, setDispenseType}) => {
//     const superVisibility = currentUser === User.Admin || currentUser === User.Manager;
//     const [isOpen, setIsOpen] = useState(false);
//     const dropdownRef = useRef<HTMLDivElement | null>(null);
    
    
//     const handleToggle = (checked: Boolean) => {
//         setDispenseType(checked ? DispenseType.LargeSmall : DispenseType.Classic);
//     }

//     const handleButton = async (state: IchibuState) => {
//         try {
//             await invoke("update_run_state", {newState: state});
//         } catch (error) {
//             console.error("Failed to set state:", error);
//         }
//     }

//     // Handle clicking outside the dropdown
//   useEffect(() => {
//     const handleClickOutside = (event: MouseEvent) => {
//       if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
//         setIsOpen(false);
//       }
//     };

//     document.addEventListener('click', handleClickOutside);

//     return () => {
//       document.removeEventListener('click', handleClickOutside);
//     };
//   }, []);

//     return (
//         <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
//       <DropdownMenuTrigger asChild>
//         <Button className="bg-slate-950 active:bg-slate-950 hover:bg-slate-950 focus:outline-none focus:ring-0 border-0 h-30" onClick={() => setIsOpen(!isOpen)}>
//           <img src={gear} alt="settings" className="h-10" />
//         </Button>
//       </DropdownMenuTrigger>

//       {/* Wrap the content in a div and attach the ref here */}
//       <DropdownMenuContent className="bg-gray-600 h-96">
//         <div ref={dropdownRef}>
//           {superVisibility && (
//             <DropdownMenuItem onClick={() => console.log('Option 1 clicked')}>
//               <div className="flex items-center space-x-2">
//                 <Switch
//                   checked={currentDispenseType === DispenseType.LargeSmall}
//                   onCheckedChange={handleToggle}
//                   id="dispense-mode"
//                 />
//                 <Label className="text-xl font-bold">Sized Dispense</Label>
//               </div>
//             </DropdownMenuItem>
//           )}
//           <DropdownMenuItem onClick={() => handleButton(IchibuState.Cleaning)}>
//             <Button className="w-full h-20">Clean Mode</Button>
//           </DropdownMenuItem>
//           <DropdownMenuItem onClick={() => handleButton(IchibuState.Emptying)}>
//             <Button className="w-full h-20 bg-blue-500">Empty Hopper Start</Button>
//           </DropdownMenuItem>
//           <DropdownMenuItem onClick={() => handleButton(IchibuState.Cleaning)}>
//             <Button className="w-full h-20 bg-destructive">Empty Hopper Stop</Button>
//           </DropdownMenuItem>
//         </div>
//       </DropdownMenuContent>
//     </DropdownMenu>
//     )
// }

// export default SettingsMenu;

