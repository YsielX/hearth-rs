local card={api_version=1,id="EX1_158",name="Soul of the Forest",text="Give your minions \"<b>Deathrattle:</b> Summon a 2/2 Treant.\"",set="EXPERT1",type="spell",class="druid",rarity="common",spell_school="nature",cost=3}
function card.on_play(ctx,self)for _,e in ipairs(ctx:friendly_minions(self))do ctx:attach_hook(e,"on_deathrattle","EX1_158");ctx:grant_keyword(e,"deathrattle")end end
function card.on_deathrattle(ctx,self,pos)ctx:summon_at(ctx:controller(self),"EX1_158t",pos)end
card.tokens={{id="EX1_158t",name="Treant",text="",set="EXPERT1",type="minion",class="druid",collectible=false,cost=1,attack=2,health=2}}
return card
