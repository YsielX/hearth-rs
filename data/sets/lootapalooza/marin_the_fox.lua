local treasures={"LOOT_998h","LOOT_998j","LOOT_998k","LOOT_998l"}
local chest={id="LOOT_357l",name="Master Chest",text="<b>Deathrattle:</b> Give your opponent a fantastic treasure!",set="LOOTAPALOOZA",type="minion",collectible=false,cost=3,attack=0,health=8,keywords={"deathrattle"}}
function chest.on_deathrattle(ctx,self)ctx:random_value(treasures,"grant_treasure")end
function chest.grant_treasure(ctx,self,id)ctx:give_card(ctx:opponent(ctx:controller(self)),id)end
local goblet={id="LOOT_998h",name="Tolin's Goblet",text="Draw a card. Fill your hand with copies of it.",set="LOOTAPALOOZA",type="spell",collectible=false,cost=3}
function goblet.on_play(ctx,self)local d=ctx:deck(ctx:controller(self));if #d==0 then ctx:draw(ctx:controller(self),1);return end;local e=d[1];ctx:draw_entity(ctx:controller(self),e);ctx:continue_with_entity("fill_goblet",e)end
function goblet.fill_goblet(ctx,self,e)local p=ctx:controller(self);for _=1,10-#ctx:hand(p) do ctx:give_copy(p,e)end end
local crown={id="LOOT_998j",name="Zarog's Crown",text="<b>Discover</b> a <b>Legendary</b> minion. Summon two copies of it.",set="LOOTAPALOOZA",type="spell",collectible=false,cost=3,keywords={"discover"}}
function crown.on_play(ctx,self)local p={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion"and d.rarity=="legendary"then p[#p+1]=id end end;if #p>0 then ctx:discover_cards(ctx:controller(self),"Discover a Legendary minion",p,3,"crown_chosen")end end
function crown.crown_chosen(ctx,self,id)local p=ctx:controller(self);ctx:summon(p,id);ctx:summon(p,id)end
local kobold={id="LOOT_998k",name="Golden Kobold",text="[x]<b>Taunt</b>\n<b> Battlecry:</b> Replace your hand\nwith <b>Legendary</b> minions.\nThey cost (1) less.",set="LOOTAPALOOZA",type="minion",collectible=false,cost=3,attack=6,health=6,keywords={"taunt","battlecry"}}
function kobold.on_battlecry(ctx,self)ctx:continue_with("replace_kobold_card")end
function kobold.replace_kobold_card(ctx,self)local target=nil;for _,e in ipairs(ctx:hand(ctx:controller(self)))do if ctx:get_data(self,"kobold_done:"..e)==0 then target=e;break end end;if not target then return end;local p={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion"and d.rarity=="legendary"then p[#p+1]={target=target,id=id}end end;if #p>0 then ctx:random_value(p,"replace_with_legendary")end end
function kobold.replace_with_legendary(ctx,self,x)cardlib.effects.transform(ctx, x.target,x.id);cardlib.effects.modify(ctx, x.target,{stat="cost",operation="add",value=-1});ctx:set_data(self,"kobold_done:"..x.target,1);ctx:continue_with("replace_kobold_card")end
local wand={id="LOOT_998l",name="Wondrous Wand",text="Draw 3 cards.\nSet their Cost to (1).",set="LOOTAPALOOZA",type="spell",collectible=false,cost=3}
function wand.on_play(ctx,self)ctx:set_data(self,"wand_left",3);ctx:continue_with("wand_draw")end
function wand.wand_draw(ctx,self)local p=ctx:controller(self);local d=ctx:deck(p);local left=ctx:get_data(self,"wand_left");if left<=0 then return end;if #d==0 then ctx:set_data(self,"wand_left",0);ctx:draw(p,left);return end;local e=d[1];ctx:draw_entity(p,e);ctx:continue_with_entity("wand_discount",e)end
function wand.wand_discount(ctx,self,e)if ctx:entity(e).zone=="hand"then cardlib.effects.modify(ctx, e,{stat="cost",operation="set",value=1})end;local n=ctx:get_data(self,"wand_left")-1;ctx:set_data(self,"wand_left",n);if n>0 then ctx:continue_with("wand_draw")end end
local card={api_version=1,id="LOOT_357",name="Marin the Fox",text="<b>Battlecry:</b> Summon a 0/8 Treasure Chest for your opponent. <i>(Break it for awesome loot!)</i>",set="LOOTAPALOOZA",type="minion",rarity="legendary",cost=8,attack=6,health=6,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)ctx:summon(ctx:opponent(ctx:controller(self)),"LOOT_357l")end
card.tokens={chest,goblet,crown,kobold,wand}
return card
