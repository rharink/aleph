export type CameraCoverageStatus = 'Tested' | 'Sample wanted' | 'Demand signal';

export type CameraCoverage = {
	rank: number;
	status: CameraCoverageStatus;
	camera: string;
	format: string;
	sample: string;
};

export const cameraCoverage: CameraCoverage[] = [
	{
		rank: 1,
		status: 'Tested',
		camera: 'SIGMA fp',
		format: 'CinemaDNG video; DNG stills',
		sample: 'In lab'
	},
	{
		rank: 2,
		status: 'Sample wanted',
		camera: 'SIGMA fp L',
		format: 'CinemaDNG video; DNG stills',
		sample: 'Needed'
	},
	{
		rank: 3,
		status: 'Sample wanted',
		camera: 'Blackmagic Cinema Camera / Production Camera',
		format: 'CinemaDNG sequences',
		sample: 'Needed'
	},
	{
		rank: 4,
		status: 'Sample wanted',
		camera: 'Blackmagic Pocket Cinema Camera / Micro Cinema Camera',
		format: 'CinemaDNG sequences',
		sample: 'Needed'
	},
	{
		rank: 5,
		status: 'Sample wanted',
		camera: 'DJI Inspire RAW with Zenmuse X5R',
		format: 'CinemaDNG sequences',
		sample: 'Needed'
	},
	{
		rank: 6,
		status: 'Sample wanted',
		camera: 'DJI Inspire 2 with Zenmuse X5S / X7',
		format: 'CinemaDNG sequences',
		sample: 'Needed'
	},
	{
		rank: 7,
		status: 'Sample wanted',
		camera: 'Kinefinity MAVO / Terra',
		format: 'CinemaDNG / KineRAW',
		sample: 'Needed'
	},
	{
		rank: 8,
		status: 'Sample wanted',
		camera: 'Leica SL / SL2-S / SL3',
		format: 'DNG stills',
		sample: 'Needed'
	},
	{
		rank: 9,
		status: 'Sample wanted',
		camera: 'Panasonic S1H / BS1H',
		format: 'External RAW workflows',
		sample: 'Needed'
	},
	{
		rank: 10,
		status: 'Sample wanted',
		camera: 'Blackmagic Pocket Cinema Camera 4K / 6K / 6K Pro',
		format: 'BRAW / ProRes; legacy CinemaDNG',
		sample: 'Needed'
	},
	{
		rank: 11,
		status: 'Demand signal',
		camera: 'Blackmagic Cinema Camera 6K Full Frame',
		format: 'BRAW / ProRes',
		sample: 'Optional'
	},
	{
		rank: 12,
		status: 'Demand signal',
		camera: 'Blackmagic URSA Mini Pro 4.6K / 12K',
		format: 'BRAW / ProRes; legacy CinemaDNG',
		sample: 'Optional'
	},
	{
		rank: 13,
		status: 'Demand signal',
		camera: 'ARRI ALEXA Mini / Mini LF',
		format: 'ARRIRAW / ProRes',
		sample: 'Optional'
	},
	{
		rank: 14,
		status: 'Demand signal',
		camera: 'ARRI ALEXA 35',
		format: 'ARRIRAW / Apple ProRes',
		sample: 'Optional'
	},
	{
		rank: 15,
		status: 'Demand signal',
		camera: 'RED KOMODO / KOMODO-X',
		format: 'REDCODE RAW',
		sample: 'Optional'
	},
	{
		rank: 16,
		status: 'Demand signal',
		camera: 'RED V-RAPTOR / DSMC3',
		format: 'REDCODE RAW',
		sample: 'Optional'
	},
	{
		rank: 17,
		status: 'Demand signal',
		camera: 'Sony FX3 / FX30',
		format: 'XAVC; external ProRes RAW',
		sample: 'Optional'
	},
	{
		rank: 18,
		status: 'Demand signal',
		camera: 'Sony FX6 / FX9',
		format: 'XAVC; external RAW',
		sample: 'Optional'
	},
	{
		rank: 19,
		status: 'Demand signal',
		camera: 'Canon C70 / C300 Mark III / C500 Mark II',
		format: 'Cinema RAW Light / XF-AVC',
		sample: 'Optional'
	},
	{
		rank: 20,
		status: 'Demand signal',
		camera: 'Z CAM E2 series',
		format: 'ZRAW / ProRes / H.265',
		sample: 'Optional'
	}
];
