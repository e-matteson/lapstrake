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
					sphere(d=1);
				}
			}
			translate([50,50,0])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=1);
				}
			}
		}
		hull()
		{
			translate([50,50,0])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=1);
				}
			}
			translate([100,20,75])
			{
				rotate(0,[0,0,1])
				{
					sphere(d=1);
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
				sphere(d=2);
			}
		}
		translate([50,50,0])
		{
			rotate(0,[0,0,1])
			{
				sphere(d=2);
			}
		}
		translate([100,20,75])
		{
			rotate(0,[0,0,1])
			{
				sphere(d=2);
			}
		}
	}
}