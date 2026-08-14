local card={api_version=1,id="LOOT_309",name="Oaken Summons",text="Gain 6 Armor.\nSummon a minion\nfrom your deck that\ncosts (4) or less.",set="LOOTAPALOOZA",type="spell",class="druid",rarity="common",spell_school="nature",cost=4}
function card.on_play(ctx,self)local p=ctx:controller(self);ctx:gain_armor(p,6);local pool={};for _,e in ipairs(ctx:deck(p))do local x=ctx:entity(e);if x.type=="minion"and x.cost<=4 then pool[#pool+1]=e end end;if #pool>0 and #ctx:board(p)<7 then ctx:random_entity(pool,"oaken_recruit")end end
function card.oaken_recruit(ctx,self,e)ctx:recruit(ctx:controller(self),e)end
return card
