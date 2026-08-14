local outcomes = { "LOOT_278t1", "LOOT_278t2", "LOOT_278t3", "LOOT_278t4" }
local function reveal(ctx, self) ctx:random_value(outcomes, "reveal_elixir") end
local card = {
    api_version = 1, id = "LOOT_278", name = "Unidentified Elixir",
    text = "Give a minion +2/+2. Gains a bonus effect in your hand.", set = "LOOTAPALOOZA",
    type = "spell", class = "priest", rarity = "common", spell_school = "holy", cost = 3,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) ctx:buff(target, 2, 2) end,
    triggers = {
        { event = "game_started", timing = "after", active_zones = { "hand" }, effect = reveal },
        { event = "card_drawn", timing = "after", active_zones = { "hand" }, condition = function(ctx, self, event) return event.entity == self end, effect = reveal },
        { event = "card_created", timing = "after", active_zones = { "hand" }, condition = function(ctx, self, event) return event.entity == self end, effect = reveal },
    },
}
function card.reveal_elixir(ctx, self, id) ctx:transform(self, id) end
local function elixir(id, name, text, school, effect)
    return { id=id, name=name, text=text, set="LOOTAPALOOZA", type="spell", class="priest", collectible=false,
        spell_school=school, cost=3, target_mode="required", targets=function(ctx) return ctx:minions() end, on_play=effect }
end
local life = elixir("LOOT_278t1", "Elixir of Life", "Give a minion +2/+2 and <b>Lifesteal</b>.", "holy", function(ctx,self,target) ctx:buff(target,2,2);ctx:grant_keyword(target,"lifesteal") end)
local purity = elixir("LOOT_278t2", "Elixir of Purity", "Give a minion +2/+2 and <b>Divine Shield</b>.", "holy", function(ctx,self,target) ctx:buff(target,2,2);ctx:grant_keyword(target,"divine_shield") end)
local shadows = elixir("LOOT_278t3", "Elixir of Shadows", "Give a minion +2/+2. Summon a 1/1 copy of\u{a0}it.", "shadow", function(ctx,self,target) ctx:buff(target,2,2);ctx:summon_copy_with_stats(ctx:controller(self),target,1,1) end)
local hope = elixir("LOOT_278t4", "Elixir of Hope", "[x]Give a minion +2/+2\nand \"<b>Deathrattle:</b> Return\nthis minion to your hand.\"", "holy", function(ctx,self,target) ctx:buff(target,2,2);ctx:attach_hook(target, "on_deathrattle","LOOT_278t4");ctx:grant_keyword(target,"deathrattle") end)
function hope.on_deathrattle(ctx, self) ctx:move(self, "hand") end
card.tokens = { life, purity, shadows, hope }
return card
