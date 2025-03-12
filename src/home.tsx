
import LogIn from './login';
import { User } from './types';

interface HomeProps{
    setUser: (user: User) => void;
}

const Home: React.FC<HomeProps> = ({setUser}) => {
   


    return (
    <div className="relative flex items-center justify-center">
        <LogIn onUpdate={setUser}></LogIn>
    </div>
    );
}

export default Home;
