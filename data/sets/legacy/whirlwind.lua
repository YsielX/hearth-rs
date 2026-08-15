return {api_version=1,id="EX1_400",name="Whirlwind",text="Deal $1 damage to ALL minions.",set="LEGACY",type="spell",class="warrior",cost=1,on_play=function(ctx)ctx:damage_all(ctx:minions(),1)end}
