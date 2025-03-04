import express from 'express';
import dotenv from 'dotenv';

import notion from './services/notion';
import pdf from './services/pdf';

dotenv.config();

const app = express();
const port = process.env.PORT || 3000;

app.use(express.json());

app.get('/', (req, res) => {
	res.send('TypeScript Express API is running');
});

app.listen(port, () => {
	console.log(`Server listening on port ${port}`);
});

app.post('/webhook/notion', async (req, res) => {

	const NOTION_API_KEY = process.env.NOTION_API_KEY;
	const NOTION_DATABASE_ID = process.env.NOTION_DATABASE_ID;
});
