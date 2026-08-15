local card={api_version=1,id="DS1_183",name="Multi-Shot",text="Deal $3 damage to two random enemy minions.",set="LEGACY",type="spell",class="hunter",cost=4,rules={can_play=function(ctx,self,current)return current and #ctx:enemy_minions(self)>=2 end}}
function card.on_play(ctx,self)local m=ctx:enemy_minions(self);local pairs={};for i=1,#m-1 do for j=i+1,#m do pairs[#pairs+1]={m[i],m[j]}end end;if #pairs>0 then ctx:random_value(pairs,"hit_pair")end end
function card.hit_pair(ctx,self,pair)ctx:damage_batch({{pair[1],3},{pair[2],3}})end
return card
