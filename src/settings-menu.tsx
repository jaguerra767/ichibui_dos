import React from 'react';
import{ Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";

import gear from '@/assets/gear-white.svg';
import { Label } from '@radix-ui/react-dropdown-menu';


import { DispenseType, IchibuState, User } from '@/types';
import { invoke } from '@tauri-apps/api/core';

interface SettingsMenuProps{
    currentUser: User,
    currentDispenseType: DispenseType
    setDispenseType: (type: DispenseType) => void;
}

const SettingsMenu: React.FC<SettingsMenuProps> = ({currentUser, currentDispenseType, setDispenseType}) => {
    const superVisibility = currentUser === User.Admin || currentUser === User.Manager;
    const handleToggle = (checked: Boolean) => {
        setDispenseType(checked ? DispenseType.LargeSmall : DispenseType.Classic);
    }

    const handleButton = async (state: IchibuState) => {
        try {
            await invoke("update_run_state", {newState: state});
        } catch (error) {
            console.error("Failed to set state:", error);
        }
    }
    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className='bg-slate-950'> 
                    <img src={gear} alt='settings' className='h-10'></img>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent className='bg-gray-600'>
                {superVisibility && <DropdownMenuItem onClick={() => console.log('Option 1 clicked')}>
                    <Switch 
                        checked={currentDispenseType === DispenseType.LargeSmall}
                        onCheckedChange={handleToggle}
                        id="dispense-mode"/>
                    <Label>Sized Dispense</Label>
                </DropdownMenuItem>}
                <DropdownMenuItem onClick={() => handleButton(IchibuState.Cleaning)}>
                    <Button className='w-full'>
                        Clean Mode
                    </Button>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleButton(IchibuState.Emptying)}>
                <Button className='w-full bg-blue-500'>
                        Empty Hopper Start
                    </Button>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleButton(IchibuState.Cleaning)}>
                <Button className='w-full bg-destructive'>
                        Empty Hopper Stop
                    </Button>
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    )
}

export default SettingsMenu;

