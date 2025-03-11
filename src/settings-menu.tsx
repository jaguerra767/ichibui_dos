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


import { DispenseType, RunState, User } from '@/types';
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

    const handleButton = async (state: RunState) => {
        try {
            await invoke("update_run_state", {state});
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
                <DropdownMenuItem onClick={() => handleButton(RunState.Cleaning)}>
                    <Button className='w-full'>
                        Clean Mode
                    </Button>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleButton(RunState.Emptying)}>
                <Button className='w-full bg-destructive'>
                        Empty Hopper
                    </Button>
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    )
}

export default SettingsMenu;

