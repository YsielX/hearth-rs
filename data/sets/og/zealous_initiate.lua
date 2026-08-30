local card={api_version=1,id="OG_158",name="Zealous Initiate",text="<b>Deathrattle:</b> Give a random friendly minion +1/+1.",set="OG",type="minion",rarity="common",cost=1,attack=1,health=1,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local p=ctx:friendly_minions(self);if #p>0 then ctx:random_entity(p,"buff_minion")end end
function card.buff_minion(ctx,self,target)cardlib.effects.buff(ctx, target,1,1)end
return card
