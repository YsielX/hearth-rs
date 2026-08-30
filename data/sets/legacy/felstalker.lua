local card={api_version=1,id="EX1_306", rarity = "free",name="Felstalker",text="<b>Battlecry:</b> Discard a random card.",set="LEGACY",type="minion",class="warlock",cost=2,attack=4,health=3,tags={"demon"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local h=ctx:hand(ctx:controller(self));if #h>0 then ctx:random_entity(h,"discard_chosen")end end
function card.discard_chosen(ctx,self,target)ctx:discard(ctx:controller(self),target)end
return card
