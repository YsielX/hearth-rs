local card={api_version=1,id="CS2_074",name="Deadly Poison",text="Give your weapon +2 Attack.",set="LEGACY",type="spell",class="rogue",spell_school="nature",cost=1,rules={can_play=function(ctx,self,current)return current and ctx:player(ctx:controller(self)).weapon~=nil end}}
function card.on_play(ctx,self)local w=ctx:player(ctx:controller(self)).weapon;if w then ctx:buff(w,2,0)end end
return card
