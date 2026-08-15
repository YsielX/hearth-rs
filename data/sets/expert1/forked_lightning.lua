local card={api_version=1,id="EX1_251",name="Forked Lightning",text="Deal $2 damage to 2 random enemy minions. <b>Overload:</b> (2)",set="EXPERT1",type="spell",class="shaman",rarity="common",spell_school="nature",cost=1,keywords={"overload"},keyword_params={overload=2},rules={can_play=function(ctx,self,current)return current and #ctx:enemy_minions(self)>=2 end}}
function card.on_play(ctx,self)local m=ctx:enemy_minions(self);local pairs={};for i=1,#m-1 do for j=i+1,#m do pairs[#pairs+1]={m[i],m[j]}end end;if #pairs>0 then ctx:random_value(pairs,"hit_pair")end end
function card.hit_pair(ctx,self,p)ctx:damage_batch({{p[1],2},{p[2],2}})end
return card
