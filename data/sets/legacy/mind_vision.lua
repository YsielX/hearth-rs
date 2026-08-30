local card={api_version=1,id="CS2_003", rarity = "free",name="Mind Vision",text="Put a copy of a random card in your opponent's hand into your hand.",set="LEGACY",type="spell",class="priest",spell_school="shadow",cost=1,rules={can_play=function(ctx,self,current)local p=ctx:opponent(ctx:controller(self));return current and #ctx:hand(p)>0 end}}
function card.on_play(ctx,self)local p=ctx:opponent(ctx:controller(self));local h=ctx:hand(p);if #h>0 then ctx:random_entity(h,"copy_selected")end end
function card.copy_selected(ctx,self,target)ctx:give_copy(ctx:controller(self),target)end
return card
