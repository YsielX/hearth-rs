local card={api_version=1,id="CS2_114", rarity = "free",name="Cleave",text="[x]Deal $2 damage to\ntwo random enemy\nminions.",set="LEGACY",type="spell",class="warrior",cost=2,rules={can_play=function(ctx,self,current)return current and #ctx:enemy_minions(self)>=2 end}}
function card.on_play(ctx,self)local m=ctx:enemy_minions(self);local pairs={};for i=1,#m-1 do for j=i+1,#m do pairs[#pairs+1]={m[i],m[j]}end end;if #pairs>0 then ctx:random_value(pairs,"hit_pair")end end
function card.hit_pair(ctx,self,pair)cardlib.effects.damage_batch(ctx, {{pair[1],2},{pair[2],2}})end
return card
