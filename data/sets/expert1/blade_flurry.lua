local card={api_version=1,id="CS2_233",name="Blade Flurry",text="Destroy your weapon and deal its damage to all enemy minions.",set="EXPERT1",type="spell",class="rogue",rarity="rare",cost=2,rules={can_play=function(ctx,self,current)return current and ctx:player(ctx:controller(self)).weapon~=nil end}}
function card.on_play(ctx,self)local w=ctx:player(ctx:controller(self)).weapon;if w then local a=ctx:entity(w).attack;ctx:destroy(w);ctx:damage_all(ctx:enemy_minions(self),a)end end
return card
