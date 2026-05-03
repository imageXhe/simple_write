<template>
    <div class="setting-panel">
        <a-popover trigger="click" :open="popover" @openChange="popoverClickChange">
            <div class="setting-trigger">
                <a-button ghost class="project_name_button" >
                    <span class="project_name_button__spacer" aria-hidden="true"></span>
                    <span class="project_name_button__center">
                        <UpOutlined />
                        <span class="project_name_button__name">{{ project_now }}</span>
                    </span>
                    <ProfileOutlined class="project_name_button__profile" />
                </a-button>
            </div>
            <template #content>
                <div v-for="item in project_arr" :key="item">
                    <a-button ghost size="small" @click="ChangeProject(item)" class="select_project">
                        {{ item.name }}
                    </a-button>  
                </div>
                <a-divider class="divider"></a-divider>
                <a-button ghost size="small" @click="Manage" class="select_project">
                    <WalletOutlined  />
                    {{ t('file.manage') }} · · ·
                </a-button>
            </template>
        </a-popover>
    </div>

    <div>
        <a-modal :title="t('file.manageWarehouse')" v-model:open="visible_manage" :maskClosable="false"
        :footer="null" class="modal" width="650px" @cancel="CancelManage">
            <a-layout>
                <a-layout-sider class="modal_sider" width="300" >
                    <a-popover trigger="click" placement="bottom">
                        <a-button style="width: 100%;" type="primary">
                            {{ t('file.newWarehouse') }}
                            <AppstoreAddOutlined />
                        </a-button>
                        <a-button style="width: 100%;margin-top: 5px;" type="primary"
                            @click.stop="OpenFolderAsWarehouse">
                            {{ t('file.openFolderAsWarehouse') }}
                            <AppstoreOutlined />
                        </a-button>
                        <template #content>
                            <a-input :placeholder="t('file.pleaseEnterName')" v-model:value="create_project_name" />
                            <a-input-group compact style="margin: 10px 0;">
                                <a-input auto-size style="width: calc(100% - 50px);" disabled
                                :placeholder="t('file.pleaseSelectDirectory')" v-model:value="create_project_path"/>
                                <a-button @click="SelectPath('create')" style="width: 50px;text-align: center;padding:0 5px">
                                    {{  t('file.select')}}
                                </a-button>
                            </a-input-group>
                            <a-button @click="CreateProject" type="primary" style="width: 100%;">
                                {{  t('file.newWarehouse') }}
                            </a-button>
                        </template>
                    </a-popover>
                    <div v-for="item in project_arr" :key="item">
                        <a-button class="modal_sider_button" @click="ProjectInfo(item)">
                            <a-typography-text strong>
                                {{ item.name }}
                            </a-typography-text>
                            <br>
                            <a > {{ item.path }}</a>    
                        </a-button>
                    </div>
                    <div v-if="warehouse_json.num==0">
                        <a-empty :description="t('file.noData')" style="margin: 100px 0;"/>
                    </div>
                </a-layout-sider>
                <a-layout-content class="modal_content">
                    <a-card style="height: 100%;" v-if="project_selected">
                        <h1 style="text-align: start; color: var(--text-primary, #262626);">{{ project_info.name }}</h1>
                        <h4 style="color: var(--text-tertiary, #8c8c8c);">{{ project_info.path }}</h4>
                        <a-divider style="margin: 10px 0;"/>
                        <a-tabs size="small" v-model:activeKey="activeKey" centered :tabBarGutter="30">
                            <!-- 重命名 -->
                            <a-tab-pane key="1" >
                                <template #tab>
                                    <a-tooltip placement="top" :title="t('file.rename')" :arrow="0">
                                        <EditOutlined class="tabBar_class"/>
                                    </a-tooltip>
                                </template>
                                <div>
                                    <a-flex justify="space-between">
                                        <h3 style="color: var(--text-primary, #262626);">{{  t('file.confirmInfo')}}</h3>
                                        <a-popconfirm
                                            :title="t('file.confirmRename')"
                                            @confirm="RenameButton"
                                            :ok-text="t('file.confirm')"
                                            :cancel-text="t('file.cancel')"
                                        >
                                            <a-button type="primary">
                                                {{ t('file.rename') }}
                                            </a-button>
                                        </a-popconfirm>
                                    </a-flex>
                                    <br>
                                    <a-input :addon-before="t('file.currentName')" disabled :value="project_info.name"/>
                                    <br><br>
                                    <a-input :addon-before="t('file.newName')" :placeholder="t('file.enter')" v-model:value="rename_project_name"/>
                                </div>
                            </a-tab-pane>
                            <!-- 移动 -->
                            <a-tab-pane key="2" >
                                <template #tab>
                                    <a-tooltip placement="top" :title="t('file.move')" :arrow="0">
                                        <SnippetsOutlined class="tabBar_class"/>
                                    </a-tooltip>
                                </template>
                                <div>
                                    <a-flex justify="space-between">
                                        <h3 style="color: var(--text-primary, #262626);">{{ t('file.confirmInfo') }}</h3>
                                        <a-popconfirm :title="tconfirmMove" @confirm="MoveButton"
                                        :ok-text="t('file.confirm')" :cancel-text="t('file.cancel')">
                                            <a-button type="primary">{{ t('file.move') }}</a-button>
                                        </a-popconfirm>
                                    </a-flex>
                                    <br>
                                    <a-input :addon-before="t('file.currentPath')" disabled 
                                    :value="project_info.path" />
                                    <br><br>
                                    <a-input :addon-before="t('file.newPath')" :placeholder="t('file.pleaseSelectDirectory')"
                                    v-model:value="move_project_path" disabled/>
                                    <br><br>
                                    <a-button @click="SelectPath('move')" type="primary" style="width: 100%;">
                                        {{ t('file.selectNewPath') }}
                                    </a-button>
                                </div>
                            </a-tab-pane>
                            <!-- 删除 -->
                            <a-tab-pane key="3" >
                                <template #tab>
                                    <a-tooltip placement="top" :title="t('file.delete')" :arrow="0">
                                        <DeleteOutlined class="tabBar_class"/>
                                    </a-tooltip>
                                </template>
                                <div>
                                    <a-flex justify="space-between">
                                        <h3 style="color: var(--text-primary, #262626);">{{ t('file.confirmInfo') }}</h3>
                                        <a-popconfirm :title="t('file.confirmDelete')" @confirm="DeleteButton"
                                        :ok-text="t('file.confirm')" :cancel-text="t('file.cancel')">
                                            <a-button type="primary">
                                                {{  t('file.delete') }}
                                            </a-button>
                                        </a-popconfirm>
                                    </a-flex>
                                    <br>
                                    <a-input :addon-before="t('file.currentName')" disabled :value="project_info.name"/>
                                    <br><br>
                                    <a-input :addon-before="t('file.currentPath')" disabled :value="project_info.path"/>
                                </div>
                            </a-tab-pane>
                        </a-tabs>
                    </a-card>
                    <a-card style="height: 100%;" v-if="!project_selected">
                        <a-empty :description="t('file.pleaseSelectWarehouse')" style="margin: 100px 0;"/>
                    </a-card>
                </a-layout-content>
            </a-layout>
        </a-modal>
    </div>
</template>

<script setup>
import { ref, onMounted  } from 'vue'
import {
    UpOutlined,
    QuestionCircleOutlined,
    SettingOutlined,
    WalletOutlined,
    AppstoreAddOutlined,
    EditOutlined,
    SnippetsOutlined,
    DeleteOutlined,
    ProfileOutlined,
    AppstoreOutlined
} from '@ant-design/icons-vue'
import { message } from 'ant-design-vue';
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { Store } from '@tauri-apps/plugin-store';
import { useI18n } from "../locales";

const { t } = useI18n();

const warehouse_json = ref()
const project_arr = ref([])

const popover = ref(false)
const popoverClickChange = () => {
    popover.value = !popover.value
}

onMounted(async() => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        warehouse_json.value = { num: 0, project_list: [] }
        project_arr.value = []
        return
    }

    invoke("init_warehouse_json").then((response) => {
        console.log(response)
    })
    get_warehouse_json()
    const store = await Store.load('store.json');
    const v = await store.get('warehouse_now');
    project_now.value = v.name;
})
const get_warehouse_json = () => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        warehouse_json.value = { num: 0, project_list: [] }
        project_arr.value = []
        return
    }

    invoke("get_warehouse_json").then(async(response) => {
        warehouse_json.value = response
        project_arr.value = response.project_list
        //console.log(warehouse_json.value)
    })
}
const activeKey = ref('0');

const project_info = ref({})
const project_selected = ref(false)
const project_now = ref("test")
const create_project_name = ref('')
const create_project_path= ref('')

const ChangeProject = async(e) => {
    //todo
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return
    }

    const store = await Store.load('store.json');
    await store.set('warehouse_now', e );
    project_now.value = e.name;
    console.log(project_now.value)
}
const OpenFolderAsWarehouse = async() => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return
    }

    const selectedFolder = await open({
        multiple: false,
        directory: true,
    });
    if (!selectedFolder) {
        return
    }

    invoke('open_warehouse', { folderPath: selectedFolder })
        .then((response) => {
            console.log(response)
            if (response.status == "SUCCESS") {
                message.success(t('file.warehouseCreated'))
                get_warehouse_json()
            } else if (response.status == "ERROR") {
                message.error(response.msg)
            }
        })
}

const SelectPath = async(e) => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return
    }

    const SelectFile = await open({
        multiple: false,
        directory: true,
    });
    if(e == 'create'){
        create_project_path.value = SelectFile
        console.log(create_project_path.value)
    }else if(e == 'move'){
        move_project_path.value = SelectFile
        console.log(move_project_path.value)
    }
}
const CreateProject = () => {
    if(create_project_name.value == '' || create_project_path.value == ''){
        message.warning(t('file.enterNameAndPath'));
        return;
    }

    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return
    }

    invoke('create_warehouse', { path: create_project_path.value, name: create_project_name.value })
        .then((response) => {
            console.log(response)
            if( response.status == "SUCCESS"){
                message.success(t('file.warehouseCreated'))
                get_warehouse_json()
            }else if( response.status == "ERROR"){
                message.error(response.msg)
                console.log(response.msg)
            }
        })
}

const visible_manage = ref(false)
const Manage = () => {
    popover.value = false
    visible_manage.value = true
}
const ProjectInfo = (e) => {
    if(e.id != project_info.value.id){
        rename_project_name.value = ''
        move_project_path.value = ''
    }
    project_info.value = e
    console.log(project_info.value)
    project_selected.value = true
}

const rename_project_name = ref('')
const RenameButton = () => {
    if(project_info.value.name == rename_project_name.value){
        message.warning(t('file.sameName'));
    }else if(rename_project_name.value == ''){
        message.warning(t('file.pleaseEnterName'));
    }else{
        if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
            return
        }

        invoke('rename_warehouse', { 
            path: project_info.value.path, 
            name: project_info.value.name, 
            id: project_info.value.id, 
            newName: rename_project_name.value 
        }).then((response) => {
            console.log(response)
            if( response.status == "SUCCESS"){
                message.success(t('message.success'))
                get_warehouse_json()
                project_info.value.name = rename_project_name.value
            }else if( response.status == "ERROR"){
                message.error(response.msg)
                console.log(response.msg)
            }
        })
    }  
}

const move_project_path= ref('')
const MoveButton = () => {
    if(project_info.value.path == move_project_path.value){
        message.warning(t('file.samePath'));
    }else if(move_project_path.value == '' || move_project_path.value == null){
        message.warning(t('file.selectNewPath'));
    }else{
        if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
            return
        }

        invoke('move_warehouse', { 
            path: project_info.value.path, 
            name: project_info.value.name, 
            id: project_info.value.id, 
            newPath: move_project_path.value 
        }).then((response) => {
            console.log(response)
            if( response.status == "SUCCESS"){
                message.success(t('message.success'))
                get_warehouse_json()
                project_info.value.path = move_project_path.value
            }else if( response.status == "ERROR"){
                message.error(response.msg)
                console.log(response.msg)
            }
        })
    }  
}

const DeleteButton = () => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return
    }

    invoke('delete_warehouse', { 
            path: project_info.value.path, 
            name: project_info.value.name, 
            id: project_info.value.id, 
        }).then((response) => {
            console.log(response)
            if( response.status == "SUCCESS"){
                message.success("t('message.success')")
                get_warehouse_json()
                project_selected.value = false
            }else if( response.status == "ERROR"){
                message.error(response.msg)
                console.log(response.msg)
            }
        })
}

const CancelManage = () => {
    setTimeout(() => {
        project_selected.value = false
        project_info.value = {}
        activeKey.value = '0';
    }, 300);
}

</script>

<style scoped>
.button_selected_not {
    color: black;
}
.setting-panel {
    width: 100%;
}
.setting-trigger {
    width: 100%;
}
.project_name_button{
    width: 100%;
    color: var(--text-primary, #262626);
    font-size: small;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0 12px;
    box-sizing: border-box;
}
.project_name_button__spacer {
    width: 100%;
}
.project_name_button__center {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    justify-self: center;
    white-space: nowrap;
}
.project_name_button__name {
    display: inline-block;
}
.project_name_button__profile {
    justify-self: end;
}
.select_project{
    width: 110px;
    color: var(--text-primary, #828282);
    font-size: 14px;
    text-align: start;
    height: 35px;
}
.modal{
    color: antiquewhite;
}
.modal_sider{
    overflow: auto;
    height: 400px;
    background-color: var(--bg-base, #fff);
}
.modal_sider_button{
    width: 100%;
    height: auto;
    margin: 5px 0 5px 0;
    color: var(--text-secondary, #595959);
    text-align: start;
    text-overflow: ellipsis;
    overflow:hidden;
}
.modal_content{
    background-color: var(--bg-base, #fff);
}

.divider{
    margin: 5px 0;
    background-color: blue
}

.tabBar_class{
    margin: 0 15px;
}
</style>
