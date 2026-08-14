local card={api_version=1,id="OG_221",name="Selfless Hero",text="<b>Deathrattle:</b> Give a random friendly minion <b>Divine Shield</b>.",set="OG",type="minion",class="paladin",rarity="rare",cost=1,attack=2,health=1,tags={"draenei"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self)local p=ctx:friendly_minions(self);if #p>0 then ctx:random_entity(p,"shield_minion") end end
function card.shield_minion(ctx,self,target)ctx:grant_keyword(target,"divine_shield")end
return card
