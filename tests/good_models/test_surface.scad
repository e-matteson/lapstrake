$fn=5;
hull()
{
	translate([0,0,0])
	{
		rotate(0,[0,0,1])
		{
			sphere(d=1);
		}
	}
	translate([10,0,0])
	{
		rotate(0,[0,0,1])
		{
			sphere(d=1);
		}
	}
	translate([10,10,0])
	{
		rotate(0,[0,0,1])
		{
			sphere(d=1);
		}
	}
}
