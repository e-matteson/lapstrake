$fn=5;
union()
{
	union()
	{
		hull()
		{
			translate([0,0,0])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=0.01);
				}
			}
			translate([50,50,0])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=0.01);
				}
			}
		}
		hull()
		{
			translate([50,50,0])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=0.01);
				}
			}
			translate([100,20,75])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=0.01);
				}
			}
		}
	}
	union()
	{
		translate([0,0,0])
		{
			rotate(0,[0,0,1])
			{
				sphere(d=0.02);
			}
		}
		translate([50,50,0])
		{
			rotate(0,[0,0,1])
			{
				sphere(d=0.02);
			}
		}
		translate([100,20,75])
		{
			rotate(0,[0,0,1])
			{
				sphere(d=0.02);
			}
		}
	}
}
