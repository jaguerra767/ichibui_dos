
import LogIn from './login';
import { User } from './types';

interface HomeProps{
    setUser: (user: User) => void;
}

const Home: React.FC<HomeProps> = ({setUser}) => {
   


    return (
    <div className="flex items-center justify-center h-screen">
        <LogIn onUpdate={setUser}></LogIn>
    </div>
    );
}

export default Home;
